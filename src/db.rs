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

/// Returns a single adjustment.
pub fn get_adjustment(connection: &mut MysqlConnection, id: u64) -> Option<Adjustment> {
    use crate::schema::adjustment::dsl::adjustment;

    adjustment
        .find(id)
        .select(Adjustment::as_select())
        .first(connection)
        .optional()
        .expect("Error loading adjustment")
}

/// Deletes the adjustment with the given ID.
pub fn delete_adjustment(connection: &mut MysqlConnection, id: u64) -> usize {
    diesel::delete(crate::schema::adjustment::table.find(id))
        .execute(connection)
        .expect("Error deleting adjustment")
}

/// Adds a new adjustment.
pub fn add_adjustment(
    connection: &mut MysqlConnection,
    adjustment_type: &AdjustmentType,
    comment: &Option<String>,
    created: &Option<NaiveDateTime>,
) -> usize {
    let new_adjustment = crate::models::NewAdjustment {
        adjustment_type_id: adjustment_type.id,
        comment: comment.clone(),
        created: *created,
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
pub fn add_time_entry(
    connection: &mut MysqlConnection,
    time: u16,
    created: Option<NaiveDateTime>,
) -> usize {
    let new_time_entry = crate::models::NewTimeEntry { time, created };

    diesel::insert_into(crate::schema::time_entry::table)
        .values(&new_time_entry)
        .execute(connection)
        .expect("Error inserting time entry")
}

/// Returns the time entry with the given ID.
pub fn get_time_entry(
    connection: &mut MysqlConnection,
    id: u64,
) -> Option<crate::models::TimeEntry> {
    use crate::schema::time_entry::dsl;

    dsl::time_entry
        .find(id)
        .select(crate::models::TimeEntry::as_select())
        .first(connection)
        .optional()
        .expect("Error loading time entry")
}

/// Deletes the time entry with the given ID.
pub fn delete_time_entry(connection: &mut MysqlConnection, id: u64) -> usize {
    diesel::delete(crate::schema::time_entry::table.find(id))
        .execute(connection)
        .expect("Error deleting time entry")
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
    let mut adjustments = get_adjustments(connection, &filter);

    // Sort the adjustments by creation date, ascending.
    adjustments.sort_by(|a, b| a.created.cmp(&b.created));

    // Retrieve the adjustment types for the given adjustments.
    let adjustment_types = get_adjustment_types_for_adjustments(connection, &adjustments);

    // Calculate the adjusted time.
    for adjustment in adjustments {
        let adjustment_type = adjustment_types
            .get(&adjustment.adjustment_type_id)
            .unwrap();
        adjusted_time += i32::from(adjustment_type.adjustment);
        // We can't go below 0 since screen time can't be negative.
        if adjusted_time < 0 {
            adjusted_time = 0;
        }
    }

    u16::try_from(adjusted_time).unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::r2d2::ConnectionManager;
    use diesel::result::Error;
    use diesel::{Connection, MysqlConnection};
    use r2d2::Pool;

    fn setup() -> Pool<ConnectionManager<MysqlConnection>> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<MysqlConnection>::new(database_url);
        Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .expect("Could not build connection pool")
    }

    #[test]
    fn test_get_adjustment_type() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Initially there are no adjustment types. None is returned.
            let adjustment_type = get_adjustment_type(conn, 1);
            assert!(adjustment_type.is_none());

            // Create an adjustment type.
            let result = add_adjustment_type(conn, "Test".to_string(), 1);

            // 1 record should have been inserted.
            assert_eq!(result, 1);

            // Retrieve the ID of the inserted adjustment type.
            let adjustment_type_id = crate::schema::adjustment_type::table
                .select(crate::schema::adjustment_type::dsl::id)
                .first::<u64>(conn)
                .unwrap();

            // Retrieve the adjustment type and check that it has the correct description and
            // adjustment.
            let adjustment_type = get_adjustment_type(conn, adjustment_type_id).unwrap();
            assert_eq!(adjustment_type.description, "Test");
            assert_eq!(adjustment_type.adjustment, 1);
            Ok(())
        });
    }

    #[test]
    fn test_get_adjustment_types() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Initially there are no adjustment types. An empty vector is returned.
            let adjustment_types = get_adjustment_types(conn, None);
            assert!(adjustment_types.is_empty());

            // Create 12 adjustment types.
            for i in 0..=11 {
                add_adjustment_type(conn, format!("Test {}", i), i - 6);
            }
            // Retrieve adjustment types without passing a limit. We should get 10 adjustment types
            // by default.
            let adjustment_types = get_adjustment_types(conn, None);
            assert_eq!(adjustment_types.len(), 10);

            // Pass a limit of 5. We should get 5 adjustment types.
            let adjustment_types = get_adjustment_types(conn, Some(5));
            assert_eq!(adjustment_types.len(), 5);

            // Pass a limit of 100. We should get 12 adjustment types.
            let adjustment_types = get_adjustment_types(conn, Some(100));
            for (i, adjustment_type) in adjustment_types.iter().enumerate() {
                // Check that all adjustment types have the correct description and adjustment.
                assert_eq!(adjustment_type.description, format!("Test {}", i));
                assert_eq!(adjustment_type.adjustment, i as i8 - 6);
            }
            Ok(())
        });
    }

    #[test]
    fn test_add_and_delete_adjustment_type() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Initially there are no adjustment types.
            let adjustment_types = get_adjustment_types(conn, None);
            assert!(adjustment_types.is_empty());

            // Try to delete a non-existing adjustment type. This should return 0 deleted rows.
            let rows_deleted = delete_adjustment_type(conn, 1);
            assert_eq!(rows_deleted, Ok(0));

            // Create an adjustment type.
            let rows_inserted = add_adjustment_type(conn, "Test".to_string(), 1);
            assert_eq!(rows_inserted, 1);

            // Now there should be 1 adjustment type.
            let adjustment_types = get_adjustment_types(conn, None);
            assert_eq!(adjustment_types.len(), 1);

            // Retrieve the created adjustment type so we know its ID and can delete it.
            let adjustment_types = get_adjustment_types(conn, Some(10));
            let last_adjustment_type = adjustment_types.last().unwrap();
            let rows_deleted = delete_adjustment_type(conn, last_adjustment_type.id);

            // 1 record should have been deleted.
            assert_eq!(rows_deleted, Ok(1));

            // Now there should be no adjustment types left.
            let adjustment_types = get_adjustment_types(conn, None);
            assert!(adjustment_types.is_empty());
            Ok(())
        });
    }

    #[test]
    fn fails_to_delete_adjustment_type_with_adjustments() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Create an adjustment type.
            add_adjustment_type(conn, "Test".to_string(), 1);

            // Retrieve the created adjustment type so we know its ID.
            let adjustment_types = get_adjustment_types(conn, Some(10));
            let adjustment_type = adjustment_types.last().unwrap();

            // Create an adjustment that references the adjustment type.
            add_adjustment(conn, &adjustment_type, &Some("Test".to_string()), &None);

            // When we now try to delete the adjustment type, we should get an error since it would
            // leave the adjustment without an adjustment type.
            let result = delete_adjustment_type(conn, adjustment_type.id);
            assert!(result.is_err());
            Ok(())
        });
    }

    #[test]
    fn test_get_adjustments() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Create 3 adjustment types.
            for i in 0..=2 {
                add_adjustment_type(conn, format!("Test {}", i), i - 1);
            }

            // Retrieve the adjustment types so we know their IDs.
            let adjustment_types = get_adjustment_types(conn, None);

            // Create 12 adjustments which reference the adjustment types and have different
            // creation dates.
            for i in 0..=11 {
                let created = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .checked_add_signed(chrono::Duration::days(i as i64))
                    .unwrap();
                add_adjustment(
                    conn,
                    &adjustment_types[i % 3],
                    &Some(format!("Test {}", i)),
                    &Some(created),
                );
            }

            // Retrieve adjustments without any filters. We should get 10 adjustments by default.
            let adjustments = get_adjustments(conn, &AdjustmentQueryFilter::default());
            assert_eq!(adjustments.len(), 10);

            // Retrieve adjustments with a limit of 5. We should get 5 adjustments.
            let adjustments = get_adjustments(
                conn,
                &AdjustmentQueryFilter {
                    limit: Some(5),
                    ..Default::default()
                },
            );
            assert_eq!(adjustments.len(), 5);

            // Filter by one of the adjustment types. We should get 4 adjustments.
            let adjustments = get_adjustments(
                conn,
                &AdjustmentQueryFilter {
                    atid: Some(adjustment_types[0].id),
                    ..Default::default()
                },
            );
            assert_eq!(adjustments.len(), 4);
            // Check that all adjustments have the correct adjustment type ID.
            for adjustment in adjustments {
                assert_eq!(adjustment.adjustment_type_id, adjustment_types[0].id);
            }

            // Filter by one of the adjustment types and a limit of 2. We should get 2 adjustments.
            let adjustments = get_adjustments(
                conn,
                &AdjustmentQueryFilter {
                    atid: Some(adjustment_types[1].id),
                    limit: Some(2),
                    ..Default::default()
                },
            );
            assert_eq!(adjustments.len(), 2);
            // Check that all adjustments have the correct adjustment type ID.
            for adjustment in adjustments {
                assert_eq!(adjustment.adjustment_type_id, adjustment_types[1].id);
            }

            // Filter by creation date. We should get 7 adjustments.
            let adjustments = get_adjustments(
                conn,
                &AdjustmentQueryFilter {
                    since: Some(
                        chrono::NaiveDate::from_ymd_opt(2023, 1, 6)
                            .unwrap()
                            .and_hms_opt(0, 0, 0)
                            .unwrap(),
                    ),
                    ..Default::default()
                },
            );
            assert_eq!(adjustments.len(), 7);
            // Check that all adjustments have a creation date after 6 january 2023.
            for adjustment in adjustments {
                assert!(
                    adjustment.created
                        >= chrono::NaiveDate::from_ymd_opt(2023, 1, 6)
                            .unwrap()
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                );
            }

            // Filter by creation date and adjustment type. We should get 3 adjustments.
            let adjustments = get_adjustments(
                conn,
                &AdjustmentQueryFilter {
                    atid: Some(adjustment_types[2].id),
                    since: Some(
                        chrono::NaiveDate::from_ymd_opt(2023, 1, 6)
                            .unwrap()
                            .and_hms_opt(0, 0, 0)
                            .unwrap(),
                    ),
                    ..Default::default()
                },
            );
            assert_eq!(adjustments.len(), 3);
            // Check that all adjustments have a creation date after 6 january 2023.
            for adjustment in &adjustments {
                assert!(
                    adjustment.created
                        >= chrono::NaiveDate::from_ymd_opt(2023, 1, 6)
                            .unwrap()
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                );
            }
            // Check that all adjustments have the correct adjustment type ID.
            for adjustment in adjustments {
                assert_eq!(adjustment.adjustment_type_id, adjustment_types[2].id);
            }

            Ok(())
        });
    }

    #[test]
    fn test_get_adjustment() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Initially there are no adjustments. None is returned.
            let adjustment = get_adjustment(conn, 1);
            assert!(adjustment.is_none());

            // Create an adjustment type.
            add_adjustment_type(conn, "Test".to_string(), 1);

            // Retrieve the created adjustment type so we know its ID.
            let adjustment_types = get_adjustment_types(conn, None);
            let adjustment_type = adjustment_types.last().unwrap();

            // Create an adjustment.
            let created = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let rows_inserted = add_adjustment(
                conn,
                adjustment_type,
                &Some("Test".to_string()),
                &Some(created),
            );
            assert_eq!(rows_inserted, 1);

            // Now there should be 1 adjustment.
            let adjustments = get_adjustments(conn, &AdjustmentQueryFilter::default());
            assert_eq!(adjustments.len(), 1);

            // Retrieve the created adjustment so we know its ID.
            let adjustment = adjustments.last().unwrap();

            // Retrieve the adjustment and check that it has the correct adjustment type ID, comment
            // and creation date.
            let adjustment = get_adjustment(conn, adjustment.id).unwrap();
            assert_eq!(adjustment.adjustment_type_id, adjustment_type.id);
            assert_eq!(adjustment.comment, Some("Test".to_string()));
            assert_eq!(adjustment.created, created);

            Ok(())
        });
    }

    #[test]
    fn test_delete_adjustment() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Try to delete a non-existing adjustment. This should return 0 deleted rows.
            let rows_deleted = delete_adjustment(conn, 1);
            assert_eq!(rows_deleted, 0);

            // Create an adjustment type and retrieve it so we know its ID.
            add_adjustment_type(conn, "Test".to_string(), 1);
            let adjustment_types = get_adjustment_types(conn, Some(10));
            let adjustment_type = adjustment_types.last().unwrap();

            // Create an adjustment and retrieve it so we know its ID.
            add_adjustment(conn, adjustment_type, &Some("Test".to_string()), &None);
            let adjustments = get_adjustments(conn, &AdjustmentQueryFilter::default());
            let adjustment = adjustments.last().unwrap();

            // Delete the adjustment. One record should have been deleted.
            let rows_deleted = delete_adjustment(conn, adjustment.id);
            assert_eq!(rows_deleted, 1);

            // Now there should be no adjustments left.
            let adjustments = get_adjustments(conn, &AdjustmentQueryFilter::default());
            assert!(adjustments.is_empty());

            Ok(())
        });
    }

    #[test]
    fn test_get_time_entries() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Initially there are no time entries. An empty vector is returned.
            let time_entries = get_time_entries(conn, None);
            assert!(time_entries.is_empty());

            // Create 12 time entries at different points in time.
            for i in 0..=11 {
                // Generate a timestamp, i days after 1 january 2023.
                let created = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .checked_add_signed(chrono::Duration::days(i as i64))
                    .unwrap();
                add_time_entry(conn, i as u16 * 15, Some(created));
            }
            // Retrieve time entries without passing a limit. We should get 10 time entries.
            let time_entries = get_time_entries(conn, None);
            assert_eq!(time_entries.len(), 10);

            // Pass a limit of 200. We should get all 12 time entries.
            let time_entries = get_time_entries(conn, Some(200));
            assert_eq!(time_entries.len(), 12);

            // Check that all time entries have the correct time.
            for (i, time_entry) in time_entries.iter().enumerate() {
                assert_eq!(time_entry.time, (11 - i) as u16 * 15);
            }
            Ok(())
        });
    }

    #[test]
    fn test_get_time_entry() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Initially there are no time entries. None is returned.
            let time_entry = get_time_entry(conn, 1);
            assert!(time_entry.is_none());

            // Create a time entry.
            let rows_inserted = add_time_entry(
                conn,
                120,
                Some(
                    NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap(),
                ),
            );
            assert_eq!(rows_inserted, 1);

            // Now there should be 1 time entry.
            let time_entries = get_time_entries(conn, None);
            assert_eq!(time_entries.len(), 1);

            // Get the ID of the created time entry.
            let time_entry_id = time_entries.first().unwrap().id;

            // Retrieve the time entry and check that it has the correct time and creation date.
            let time_entry = get_time_entry(conn, time_entry_id).unwrap();
            assert_eq!(time_entry.time, 120);
            assert_eq!(
                time_entry.created,
                NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            Ok(())
        });
    }

    #[test]
    fn test_add_and_delete_time_entry() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Initially there are no time entries.
            let time_entries = get_time_entries(conn, None);
            assert!(time_entries.is_empty());

            // Add a time entry.
            let rows_inserted = add_time_entry(
                conn,
                120,
                Some(
                    NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap(),
                ),
            );
            assert_eq!(rows_inserted, 1);

            // Now there should be 1 time entry.
            let time_entries = get_time_entries(conn, None);
            assert_eq!(time_entries.len(), 1);

            // Check that the time entry has the correct time and creation date.
            let time_entry = time_entries.last().unwrap();
            assert_eq!(time_entry.time, 120);
            assert_eq!(
                time_entry.created,
                NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );

            // Delete the time entry.
            delete_time_entry(conn, time_entry.id);

            // Now there should be no time entries left.
            let time_entries = get_time_entries(conn, None);
            assert!(time_entries.is_empty());

            Ok(())
        });
    }

    #[test]
    fn test_get_adjusted_time() {
        let pool = setup();
        let mut conn = pool.get().unwrap();
        conn.test_transaction::<_, Error, _>(|conn| {
            // Initially there are no time entries nor adjustments. The adjusted time should be 0.
            let adjusted_time = get_adjusted_time(conn);
            assert_eq!(adjusted_time, 0);

            // Create 2 adjustment types. One with a positive adjustment and one with a negative
            // adjustment.
            add_adjustment_type(conn, "Cleaned room".to_string(), 2);
            add_adjustment_type(conn, "Late in bed".to_string(), -1);

            // Retrieve the adjustment types so we know their IDs.
            let adjustment_types = get_adjustment_types(conn, None);
            let positive_adjustment_type = adjustment_types.first().unwrap();
            let negative_adjustment_type = adjustment_types.last().unwrap();

            // Create a negative adjustment. This should not affect the adjusted time since we
            // can't go below 0.
            // For every adjustment created we increase the created date by 1 second so we can
            // check that subsequent time entries override previous adjustments.
            let mut created =
                NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
            add_adjustment(conn, negative_adjustment_type, &None, &Some(created));
            let adjusted_time = get_adjusted_time(conn);
            assert_eq!(adjusted_time, 0);

            // Create an anonymous function to increase the created date by 1 second, by reference.
            let add_1_second = |created: &mut NaiveDateTime| {
                *created = created
                    .checked_add_signed(chrono::Duration::seconds(1))
                    .unwrap();
            };

            // Create a positive adjustment. This should increase the adjusted time.
            add_1_second(&mut created);
            add_adjustment(conn, positive_adjustment_type, &None, &Some(created));
            let adjusted_time = get_adjusted_time(conn);
            assert_eq!(adjusted_time, 2);

            // Create a few more positive and negative adjustments.
            add_1_second(&mut created);
            add_adjustment(conn, positive_adjustment_type, &None, &Some(created));
            add_1_second(&mut created);
            add_adjustment(conn, negative_adjustment_type, &None, &Some(created));
            add_1_second(&mut created);
            add_adjustment(conn, positive_adjustment_type, &None, &Some(created));
            let adjusted_time = get_adjusted_time(conn);
            assert_eq!(adjusted_time, 5);

            // Create a time entry. This should override all previous adjustments.
            add_1_second(&mut created);
            add_time_entry(conn, 120, Some(created));
            let adjusted_time = get_adjusted_time(conn);
            assert_eq!(adjusted_time, 120);

            // Do a few more adjustments.
            add_1_second(&mut created);
            add_adjustment(conn, negative_adjustment_type, &None, &Some(created));
            assert_eq!(get_adjusted_time(conn), 119);

            add_1_second(&mut created);
            add_adjustment(conn, positive_adjustment_type, &None, &Some(created));
            assert_eq!(get_adjusted_time(conn), 121);

            Ok(())
        });
    }
}
