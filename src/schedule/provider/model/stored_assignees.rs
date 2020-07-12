//! Model for serialized scheduled assignees in persistent storage

use diesel::deserialize::{FromSql, Result as FromSqlResult};
use diesel::serialize::{Output, Result as ToSqlResult, ToSql};
use diesel::{backend::Backend, sql_types::Text};
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Model for serialized scheduled assignees in persistent storage
#[derive(AsExpression, Debug, Deserialize, FromSqlRow, PartialEq, Serialize)]
#[sql_type = "Text"]
pub struct StoredAssignees(pub Vec<i32>);

impl<TDatabase: Backend> ToSql<Text, TDatabase> for StoredAssignees
where
    String: ToSql<Text, TDatabase>,
{
    /// Converts this model to a SQL type by serializing it as a JSON array of numbers
    fn to_sql<W: Write>(&self, out: &mut Output<W, TDatabase>) -> ToSqlResult {
        (serde_json::to_string(&self.0)?).to_sql(out)
    }
}

impl<TDatabase: Backend> FromSql<Text, TDatabase> for StoredAssignees
where
    String: FromSql<Text, TDatabase>,
{
    /// Creates this model from a SQL type
    /// The data is serialized as JSON for easy deserialization
    fn from_sql(bytes: Option<&TDatabase::RawValue>) -> FromSqlResult<Self> {
        Ok(serde_json::from_str(&String::from_sql(bytes)?)?)
    }
}
