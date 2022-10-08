//! Model for serialized weekly times in persistent storage

use crate::schedule::model::WeeklyTimes;
use diesel::deserialize::{FromSql, Result as FromSqlResult};
use diesel::serialize::{Output, Result as ToSqlResult, ToSql};
use diesel::{backend::Backend, sql_types::Text};
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Model for serialized weekly times in persistent storage
#[derive(AsExpression, Debug, Deserialize, Eq, FromSqlRow, PartialEq, Serialize)]
#[sql_type = "Text"]
pub struct StoredWeeklyTimes(pub WeeklyTimes);

impl<TDatabase: Backend> ToSql<Text, TDatabase> for StoredWeeklyTimes
where
    String: ToSql<Text, TDatabase>,
{
    /// Converts this model to a SQL type
    /// Data is serialized as a JSON object of weekday names to arrays of time strings
    fn to_sql<W: Write>(&self, out: &mut Output<W, TDatabase>) -> ToSqlResult {
        (serde_json::to_string(&self.0)?).to_sql(out)
    }
}

impl<TDatabase: Backend> FromSql<Text, TDatabase> for StoredWeeklyTimes
where
    String: FromSql<Text, TDatabase>,
{
    /// Creates this model from a SQL type
    /// Data is serialized as a JSON object for easy deserialization
    fn from_sql(bytes: Option<&TDatabase::RawValue>) -> FromSqlResult<Self> {
        Ok(serde_json::from_str(&String::from_sql(bytes)?)?)
    }
}
