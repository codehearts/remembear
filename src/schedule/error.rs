//! Error types for scheduling

use chrono::{DateTime, Utc};
use thiserror::Error;

/// Scheduling errors
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// The starting week for a schedule is invalid
    #[error("Invalid starting week: week {week} of year {year}")]
    InvalidStartWeek {
        /// The week number of the invalid starting week
        week: u32,
        /// The year of the invalid starting week
        year: i32,
    },
    /// The starting week for a schedule is ambiguous
    #[error("Ambiguous starting week {week} of year {year}, could be {first} or {second}")]
    AmbiguousStartWeek {
        /// The week number of the invalid starting week
        week: u32,
        /// The year of the invalid starting week
        year: i32,
        /// The first possible `DateTime`
        first: DateTime<Utc>,
        /// The second possible `DateTime`
        second: DateTime<Utc>,
    },
    /// The year for a schedule's starting week is too large
    #[error("Year {0} for starting week is too large")]
    InvalidStartWeekYear(i32),
    /// The week of the year for a schedule's starting week is too large
    #[error("Week {0} of the year is too large, should be between 1 and 53 inclusive")]
    InvalidStartWeekWeek(i32),
}
