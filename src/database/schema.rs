//! Diesel-generated schemas for database tables

table! {
    /// Records for users of the service
    users (uid) {
        /// Unique identifier for the user record
        uid -> Integer,
        /// Preferred name of the user
        name -> Text,
    }
}
