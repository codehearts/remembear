//! Data model for an updated reminder

use crate::Schedule;

/// Necessary data to update an existing reminder
#[derive(Debug, PartialEq)]
pub struct UpdatedReminder {
    /// Unique identifier of the record to update
    pub uid: i32,
    /// Updated name for the reminder
    pub name: String,
    /// Updated schedule for the reminder
    pub schedule: Schedule,
}
