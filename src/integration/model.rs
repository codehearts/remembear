//! Data models for external service integrations

use crate::database::schema::integrations;
use serde::Serialize;

/// Type of UID for an integration record
#[derive(Debug, Eq, PartialEq)]
pub enum Uid {
    /// User UID
    User(i32),
}

impl Uid {
    /// Provides the UID value
    #[must_use]
    pub fn uid(&self) -> i32 {
        match self {
            Self::User(uid) => *uid,
        }
    }

    /// Provides the type of UID represented by this value
    #[must_use]
    pub fn r#type(&self) -> &'static str {
        match self {
            Self::User(_) => "user",
        }
    }
}

/// Integration configuration record for a specific UID
#[derive(AsChangeset, Debug, Insertable, Eq, PartialEq, Queryable, Serialize)]
#[primary_key("uid", "uid_type", "name")]
#[table_name = "integrations"]
pub struct Record {
    /// The UID the record applies to
    pub uid: i32,
    /// The type of entity the UID represents
    pub uid_type: &'static str,
    /// The name of the integration
    pub name: &'static str,
    /// Integration-specific stored data
    pub data: String,
}
