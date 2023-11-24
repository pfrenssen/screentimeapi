use crate::models::{Adjustment, AdjustmentType};
use diesel::{
    Connection, ExpressionMethods, MysqlConnection, OptionalExtension, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use dotenvy::dotenv;
use serde::Deserialize;
use std::env;

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

/// Returns a single adjustment type.
pub fn get_adjustment_type(atid: u64) -> Option<AdjustmentType> {
    use crate::schema::adjustment_type::dsl::adjustment_type;

    let connection = &mut establish_connection();
    adjustment_type
        .find(atid)
        .select(AdjustmentType::as_select())
        .first(connection)
        .optional()
        .expect("Error loading adjustment type")
}

/// Returns a list of adjustment types.
pub fn get_adjustment_types(limit: Option<u8>) -> Vec<AdjustmentType> {
    use crate::schema::adjustment_type::dsl::adjustment_type;

    let connection = &mut establish_connection();
    adjustment_type
        .limit(i64::from(limit.unwrap_or(10)))
        .select(AdjustmentType::as_select())
        .load(connection)
        .expect("Error loading adjustment types")
}

/// Adds a new adjustment type.
/// Returns the number of inserted rows.
pub fn add_adjustment_type(description: String, adjustment: i8) -> usize {
    let connection = &mut establish_connection();
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
pub fn delete_adjustment_type(id: u64) -> Result<usize, String> {
    let connection = &mut establish_connection();

    // Check if there are still adjustments referencing this adjustment type.
    let adjustments = get_adjustments(&AdjustmentQueryFilter {
        limit: None,
        atid: Some(id),
    });
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
#[derive(Deserialize)]
pub struct AdjustmentQueryFilter {
    // The number of adjustments to return. Defaults to 10.
    pub limit: Option<u8>,
    // Optionally filter by adjustment type ID.
    #[serde(rename(deserialize = "type"))]
    pub atid: Option<u64>,
}

/// Returns a list of adjustments.
pub fn get_adjustments(filter: &AdjustmentQueryFilter) -> Vec<Adjustment> {
    use crate::schema::adjustment::dsl;

    let connection = &mut establish_connection();
    let mut query = dsl::adjustment.into_boxed();

    // Optionally filter by adjustment type ID.
    if let Some(at_id) = filter.atid {
        query = query.filter(dsl::adjustment_type_id.eq(at_id));
    }
    query
        .limit(i64::from(filter.limit.unwrap_or(10)))
        .order(dsl::created.desc())
        .select(Adjustment::as_select())
        .load(connection)
        .expect("Error loading adjustments")
}

/// Adds a new adjustment.
pub fn add_adjustment(adjustment_type: &AdjustmentType, comment: &Option<String>) -> usize {
    let connection = &mut establish_connection();
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
pub fn get_current_time_entry() -> Option<crate::models::TimeEntry> {
    use crate::schema::time_entry::dsl;

    let connection = &mut establish_connection();
    dsl::time_entry
        .order(dsl::created.desc())
        .select(crate::models::TimeEntry::as_select())
        .first(connection)
        .optional()
        .expect("Error loading time entry")
}

/// Returns a list of time entries.
pub fn get_time_entries(limit: Option<u8>) -> Vec<crate::models::TimeEntry> {
    use crate::schema::time_entry::dsl;

    let connection = &mut establish_connection();
    dsl::time_entry
        .limit(i64::from(limit.unwrap_or(10)))
        .order(dsl::created.desc())
        .select(crate::models::TimeEntry::as_select())
        .load(connection)
        .expect("Error loading time entries")
}

/// Adds a new time entry.
pub fn add_time_entry(time: u16) -> usize {
    let connection = &mut establish_connection();
    let new_time_entry = crate::models::NewTimeEntry { time };

    diesel::insert_into(crate::schema::time_entry::table)
        .values(&new_time_entry)
        .execute(connection)
        .expect("Error inserting time entry")
}
