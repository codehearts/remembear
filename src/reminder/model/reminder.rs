//! Data model for a reminder

use crate::database::schema::reminders;
use crate::{schedule, Schedule};
use diesel::backend::Backend;
use diesel::deserialize::{FromSqlRow, Queryable, Result as FromSqlResult};
use diesel::sql_types::{Integer, Text};
use serde::Serialize;
use std::convert::TryInto;

/// Record for an individual reminder
#[derive(Debug, Identifiable, PartialEq, Serialize)]
#[primary_key("uid")]
pub struct Reminder {
    /// Unique identifier for the reminder record
    pub uid: i32,
    /// Name of the reminder
    pub name: String,
    /// Schedule for the reminder
    #[serde(flatten)]
    pub schedule: Schedule,
}

impl<TDatabase> FromSqlRow<(Integer, Text, Text, Integer, Text), TDatabase> for Reminder
where
    TDatabase: Backend,
    i32: FromSqlRow<Integer, TDatabase>,
    String: FromSqlRow<Text, TDatabase>,
    schedule::Provider: FromSqlRow<(Text, Integer, Text), TDatabase>,
{
    const FIELDS_NEEDED: usize = 5;

    /// Converts a `SQLite` row to a `Reminder` using `schedule::Provider`
    fn build_from_row<TRow: diesel::row::Row<TDatabase>>(row: &mut TRow) -> FromSqlResult<Self> {
        Ok(Self {
            uid: i32::build_from_row(row)?,
            name: String::build_from_row(row)?,
            schedule: schedule::Provider::build_from_row(row)?.try_into()?,
        })
    }
}

impl Queryable<reminders::SqlType, diesel::sqlite::Sqlite> for Reminder {
    type Row = Reminder;

    fn build(row: Self::Row) -> Self {
        row
    }
}
