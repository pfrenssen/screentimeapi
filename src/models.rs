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

#[derive(Associations, Queryable, Selectable, Tabled)]
#[diesel(table_name = crate::schema::adjustment)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(belongs_to(AdjustmentType))]
pub struct Adjustment {
    pub id: u64,
    pub adjustment_type_id: u64,
    pub created: chrono::NaiveDateTime,
    #[tabled(display_with = "display_optional_string")]
    pub comment: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::adjustment)]
pub struct NewAdjustment<'a> {
    pub adjustment_type_id: u64,
    pub comment: Option<&'a str>,
}

fn display_optional_string(o: &Option<String>) -> String {
    match o {
        Some(s) => s.clone(),
        None => String::from(""),
    }
}