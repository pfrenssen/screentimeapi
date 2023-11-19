use diesel::{Connection, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use dotenvy::dotenv;
use std::env;
use crate::models::{Adjustment, AdjustmentType};

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Returns a list of adjustment types.
pub fn get_adjustment_types(limit: Option<u8>) -> Vec<AdjustmentType> {
    use crate::schema::adjustment_type::dsl::*;

    let connection = &mut establish_connection();
    adjustment_type
        .limit(limit.unwrap_or(10) as i64)
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

/// Returns a list of adjustments.
pub fn get_adjustments(limit: Option<u8>, at_id: Option<u64>) -> Vec<Adjustment> {
    use crate::schema::adjustment::dsl;

    let connection = &mut establish_connection();
    let mut query = dsl::adjustment.into_boxed();

    // Optionally filter by adjustment type ID.
    if let Some(at_id) = at_id {
        query = query.filter(dsl::adjustment_type_id.eq(at_id));
    }
    query
        .limit(limit.unwrap_or(10) as i64)
        .order(dsl::created.desc())
        .select(Adjustment::as_select())
        .load(connection)
        .expect("Error loading adjustments")
}

/// Adds a new adjustment.
pub fn add_adjustment(adjustment_type: &AdjustmentType, comment: Option<&str>) -> usize {
    let connection = &mut establish_connection();
    let new_adjustment = crate::models::NewAdjustment {
        adjustment_type_id: adjustment_type.id,
        comment,
    };

    diesel::insert_into(crate::schema::adjustment::table)
        .values(&new_adjustment)
        .execute(connection)
        .expect("Error inserting adjustment")
}