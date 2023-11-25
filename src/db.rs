use crate::models::{Adjustment, AdjustmentType};
use chrono::NaiveDateTime;
use diesel::r2d2::ConnectionManager;
use diesel::{
    ExpressionMethods, MysqlConnection, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
};
use dotenvy::dotenv;
use r2d2::Pool;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::env;

pub fn get_connection_pool() -> Pool<ConnectionManager<MysqlConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

/// Returns a single adjustment type.
pub fn get_adjustment_type(connection: &mut MysqlConnection, atid: u64) -> Option<AdjustmentType> {
    use crate::schema::adjustment_type::dsl::adjustment_type;

    adjustment_type
        .find(atid)
        .select(AdjustmentType::as_select())
        .first(connection)
        .optional()
        .expect("Error loading adjustment type")
}

/// Returns a list of adjustment types.
pub fn get_adjustment_types(
    connection: &mut MysqlConnection,
    limit: Option<u8>,
) -> Vec<AdjustmentType> {
    use crate::schema::adjustment_type::dsl::adjustment_type;

    adjustment_type
        .limit(i64::from(limit.unwrap_or(10)))
        .select(AdjustmentType::as_select())
        .load(connection)
        .expect("Error loading adjustment types")
}

/// Adds a new adjustment type.
/// Returns the number of inserted rows.
pub fn add_adjustment_type(
    connection: &mut MysqlConnection,
    description: String,
    adjustment: i8,
) -> usize {
    let new_adjustment_type = crate::models::NewAdjustmentType {
        description,
        adjustment,
    };

    diesel::insert_into(crate::schema::adjustment_type::table)
        .values(&new_adjustment_type)
        .execute(connection)
        .expect("Error inserting adjustment type")
}

/// Deletes the adjustment type with the given ID.
/// If there are still adjustments referencing this adjustment type, the deletion will fail.
/// Todo: return a proper error type.
pub fn delete_adjustment_type(connection: &mut MysqlConnection, id: u64) -> Result<usize, String> {
    // Check if there are still adjustments referencing this adjustment type.
    let filter = AdjustmentQueryFilter {
        atid: Some(id),
        ..Default::default()
    };
    let adjustments = get_adjustments(connection, &filter);
    if !adjustments.is_empty() {
        return Err(format!(
            "There are still adjustments referencing adjustment type {id}"
        ));
    }

    let result = diesel::delete(crate::schema::adjustment_type::table.find(id)).execute(connection);
    match result {
        Ok(rows_deleted) => Ok(rows_deleted),
        Err(e) => Err(format!("Error deleting adjustment type: {e}")),
    }
}

/// A filter for the `get_adjustments()` function.
#[derive(Default, Deserialize)]
pub struct AdjustmentQueryFilter {
    // The number of adjustments to return. Defaults to 10.
    pub limit: Option<u8>,
    // Optionally filter by adjustment type ID.
    #[serde(rename(deserialize = "type"))]
    pub atid: Option<u64>,
    pub since: Option<NaiveDateTime>,
}

/// Returns a list of adjustments.
pub fn get_adjustments(
    connection: &mut MysqlConnection,
    filter: &AdjustmentQueryFilter,
) -> Vec<Adjustment> {
    use crate::schema::adjustment::dsl;

    let mut query = dsl::adjustment.into_boxed();

    // Optionally filter by adjustment type ID.
    if let Some(at_id) = filter.atid {
        query = query.filter(dsl::adjustment_type_id.eq(at_id));
    }

    // Optionally filter by `since` date.
    if let Some(since) = filter.since {
        query = query.filter(dsl::created.ge(since));
    }

    query
        .limit(i64::from(filter.limit.unwrap_or(10)))
        .order(dsl::created.desc())
        .select(Adjustment::as_select())
        .load(connection)
        .expect("Error loading adjustments")
}

/// Adds a new adjustment.
pub fn add_adjustment(
    connection: &mut MysqlConnection,
    adjustment_type: &AdjustmentType,
    comment: &Option<String>,
) -> usize {
    let new_adjustment = crate::models::NewAdjustment {
        adjustment_type_id: adjustment_type.id,
        comment: comment.clone(),
    };

    diesel::insert_into(crate::schema::adjustment::table)
        .values(&new_adjustment)
        .execute(connection)
        .expect("Error inserting adjustment")
}

/// Returns the current time entry.
pub fn get_current_time_entry(
    connection: &mut MysqlConnection,
) -> Option<crate::models::TimeEntry> {
    use crate::schema::time_entry::dsl;

    dsl::time_entry
        .order(dsl::created.desc())
        .select(crate::models::TimeEntry::as_select())
        .first(connection)
        .optional()
        .expect("Error loading time entry")
}

/// Returns a list of time entries.
pub fn get_time_entries(
    connection: &mut MysqlConnection,
    limit: Option<u8>,
) -> Vec<crate::models::TimeEntry> {
    use crate::schema::time_entry::dsl;

    dsl::time_entry
        .limit(i64::from(limit.unwrap_or(10)))
        .order(dsl::created.desc())
        .select(crate::models::TimeEntry::as_select())
        .load(connection)
        .expect("Error loading time entries")
}

/// Adds a new time entry.
pub fn add_time_entry(connection: &mut MysqlConnection, time: u16) -> usize {
    let new_time_entry = crate::models::NewTimeEntry { time };

    diesel::insert_into(crate::schema::time_entry::table)
        .values(&new_time_entry)
        .execute(connection)
        .expect("Error inserting time entry")
}

pub fn get_adjusted_time(connection: &mut MysqlConnection) -> u16 {
    // Get the most recent time entry.
    let time_entry = get_current_time_entry(connection);

    // If there is no time entry, start calculating from 0.
    let mut adjusted_time: i32 = match &time_entry {
        None => 0,
        Some(time_entry) => i32::from(time_entry.time),
    };

    // Retrieve all adjustments that were created since the most recent time entry. If we don't have
    // a time entry, yet retrieve all adjustments.
    let filter = match &time_entry {
        None => AdjustmentQueryFilter::default(),
        Some(time_entry) => AdjustmentQueryFilter {
            since: Some(time_entry.created),
            ..Default::default()
        },
    };
    let adjustments = get_adjustments(connection, &filter);

    // Retrieve the adjustment types for the given adjustments.
    let adjustment_types = get_adjustment_types_for_adjustments(connection, &adjustments);

    // Calculate the adjusted time.
    for adjustment in adjustments {
        let adjustment_type = adjustment_types
            .get(&adjustment.adjustment_type_id)
            .unwrap();
        adjusted_time += i32::from(adjustment_type.adjustment);
    }

    // Convert the adjusted time to a u16. If it is lower than 0, return 0.
    if adjusted_time < 0 {
        0
    } else {
        u16::try_from(adjusted_time).unwrap()
    }
}

/// Returns a map of adjustment types that correspond to the given adjustments.
pub fn get_adjustment_types_for_adjustments(
    connection: &mut MysqlConnection,
    adjustments: &[Adjustment],
) -> HashMap<u64, AdjustmentType> {
    // Get a list of unique adjustment type IDs from the given adjustments.
    let adjustment_type_ids: HashSet<u64> =
        adjustments.iter().map(|a| a.adjustment_type_id).collect();

    // Fetch the adjustment types for the given adjustment type IDs.
    let adjustment_types = crate::schema::adjustment_type::table
        .filter(crate::schema::adjustment_type::dsl::id.eq_any(adjustment_type_ids))
        .select(AdjustmentType::as_select())
        .load(connection)
        .expect("Error loading adjustment types");

    // Create a map of adjustment type IDs to adjustment types.
    adjustment_types.into_iter().map(|at| (at.id, at)).collect()
}
