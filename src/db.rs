use diesel::{Connection, MysqlConnection};
use dotenvy::dotenv;
use std::env;
use crate::models::AdjustmentType;

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Returns a list of adjustment types.
pub fn get_adjustment_types(limit: Option<u8>) -> Vec<AdjustmentType> {
    use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
    use crate::schema::adjustment_type::dsl::*;

    let connection = &mut establish_connection();
    adjustment_type
        .limit(limit.unwrap_or(10) as i64)
        .select(AdjustmentType::as_select())
        .load(connection)
        .expect("Error loading adjustment types")
}
