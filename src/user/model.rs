//! Data models for users of the service

use crate::database::schema::users;

/// Record for an individual user of the service
#[derive(Debug, Queryable, PartialEq)]
pub struct User {
    /// Unique identifier for the user record
    pub uid: i32,
    /// Preferred name of the user
    pub name: String,
}

/// Necessary data to create a new user
#[derive(Debug, Insertable, PartialEq)]
#[table_name = "users"]
pub struct NewUser {
    /// Preferred name of the user
    pub name: String,
}
