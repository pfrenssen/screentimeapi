use diesel::prelude::*;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Queryable, Selectable, Serialize, Tabled)]
#[diesel(table_name = crate::schema::adjustment_type)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct AdjustmentType {
    pub id: u64,
    pub description: String,
    pub adjustment: i8,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::adjustment_type)]
pub struct NewAdjustmentType {
    pub description: String,
    pub adjustment: i8,
}

#[derive(Associations, Debug, Queryable, Selectable, Serialize, Tabled)]
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

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::adjustment)]
pub struct NewAdjustment {
    #[serde(rename(deserialize = "type"))]
    pub adjustment_type_id: u64,
    pub comment: Option<String>,
    pub created: Option<chrono::NaiveDateTime>,
}

/// Represents a time entry in the database.
///
/// It has three public fields:
/// - `id` of type `u64`, which is the unique identifier of the time entry.
/// - `time` of type `u16`, which represents the total number of minutes.
/// - `created` of type `chrono::NaiveDateTime`, which is the timestamp when the time entry was created.
#[derive(Debug, Queryable, Selectable, Tabled)]
#[diesel(table_name = crate::schema::time_entry)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct TimeEntry {
    pub id: u64,
    #[tabled(display_with = "format_time")]
    pub time: u16,
    pub created: chrono::NaiveDateTime,
}

/// Formats a number of minutes into a string in the format "hh:mm".
///
/// This function is used to format the `time` field of a `TimeEntry` into a human-readable string.
/// It is passed by reference to the `display_with` attribute of the `tabled` macro.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn format_time(time: &u16) -> String {
    format!("{:01}:{:02}", time / 60, time % 60)
}

impl TimeEntry {
    /// Returns the `time` field as a formatted string.
    ///
    /// The `time` field represents the total number of minutes.
    /// This method converts it into a string in the format "hh:mm".
    ///
    /// # Examples
    ///
    /// ```
    /// let entry = TimeEntry { id: 1, time: 90, created: chrono::NaiveDateTime::from_timestamp(0, 0) };
    /// assert_eq!(entry.get_formatted_time(), "01:30");
    /// ```
    #[must_use]
    pub fn get_formatted_time(&self) -> String {
        format_time(&self.time)
    }
}

// Provide an additional field with the human-readable time string when serializing a TimeEntry.
impl Serialize for TimeEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("TimeEntry", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("time", &self.time)?;
        state.serialize_field("created", &self.created)?;
        state.serialize_field("time_formatted", &self.get_formatted_time())?;
        state.end()
    }
}

use std::fmt;

impl fmt::Display for TimeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_formatted_time())
    }
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::time_entry)]
pub struct NewTimeEntry {
    pub time: u16,
    pub created: Option<chrono::NaiveDateTime>,
}

fn display_optional_string(o: &Option<String>) -> String {
    match o {
        Some(s) => s.clone(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_formatted_time_returns_correct_format_for_full_hours() {
        let entry = TimeEntry {
            id: 1,
            time: 120,
            created: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        };
        assert_eq!(entry.get_formatted_time(), "2:00");
    }

    #[test]
    fn get_formatted_time_returns_correct_format_for_partial_hours() {
        let entry = TimeEntry {
            id: 1,
            time: 90,
            created: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        };
        assert_eq!(entry.get_formatted_time(), "1:30");
    }

    #[test]
    fn get_formatted_time_returns_correct_format_for_zero_minutes() {
        let entry = TimeEntry {
            id: 1,
            time: 0,
            created: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        };
        assert_eq!(entry.get_formatted_time(), "0:00");
    }

    #[test]
    fn get_formatted_time_returns_correct_format_for_single_digit_minutes() {
        let entry = TimeEntry {
            id: 1,
            time: 9,
            created: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        };
        assert_eq!(entry.get_formatted_time(), "0:09");
    }

    #[test]
    fn get_formatted_time_returns_correct_format_for_single_digit_hours() {
        let entry = TimeEntry {
            id: 1,
            time: 65,
            created: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        };
        assert_eq!(entry.get_formatted_time(), "1:05");
    }
}
