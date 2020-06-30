//! Diesel-generated schemas for database tables

table! {
    /// Records for active reminders
    reminders (uid) {
        /// Unique identifier for the reminder record
        uid -> Integer,
        /// Name of the reminder
        name -> Text,
        /// JSON object of day name to an array of the times of day
        schedule -> Text,
        /// Beginning of the week in which the schedule started
        #[sql_name = "startweek"]
        start_week -> Integer,
        /// JSON array of integer user uids, in order of assignment
        assignees -> Text,
    }
}

table! {
    /// Records for users of the service
    users (uid) {
        /// Unique identifier for the user record
        uid -> Integer,
        /// Preferred name of the user
        name -> Text,
    }
}

allow_tables_to_appear_in_same_query!(reminders, users);
