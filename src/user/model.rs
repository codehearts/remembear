//! Data models for users of the service

/// Record for an individual user of the service
#[derive(Queryable)]
pub struct User {
    /// Unique identifier for the user record
    pub uid: i32,
    /// Preferred name of the user
    pub name: String,
}
