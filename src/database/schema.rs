//! Diesel-generated schemas for database tables

table! {
    users (uid) {
        uid -> Integer,
        name -> Text,
    }
}
