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
pub fn add_adjustment_type(description: &str, adjustment: i8) -> usize {
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
pub fn get_adjustments(limit: Option<u8>) -> Vec<Adjustment> {
    use crate::schema::adjustment::dsl::*;

    let connection = &mut establish_connection();
    adjustment
        .limit(limit.unwrap_or(10) as i64)
        .order(created.desc())
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