//! Data models for users of the service

use crate::database::schema::users;
use serde::Serialize;

/// Record for an individual user of the service
#[derive(AsChangeset, Debug, Eq, PartialEq, Queryable, Serialize)]
#[primary_key("uid")]
pub struct User {
    /// Unique identifier for the user record
    pub uid: i32,
    /// Preferred name of the user
    pub name: String,
}

/// Necessary data to create a new user
#[derive(Debug, Insertable, Eq, PartialEq)]
#[table_name = "users"]
pub struct NewUser {
    /// Preferred name of the user
    pub name: String,
}

/// Necessary data to update a new user
pub type UpdatedUser = User;
