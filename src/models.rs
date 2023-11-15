use diesel::prelude::*;
use tabled::Tabled;

#[derive(Queryable, Selectable, Tabled)]
#[diesel(table_name = crate::schema::adjustment_type)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct AdjustmentType {
    pub id: u64,
    pub description: String,
    pub adjustment: i8,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::adjustment_type)]
pub struct NewAdjustmentType<'a> {
    pub description: &'a str,
    pub adjustment: i8,
}