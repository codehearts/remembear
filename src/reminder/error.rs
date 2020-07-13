//! Error types for reminder operations

use diesel::result::Error as DieselError;
use thiserror::Error;

/// Reminder operation errors
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// A reminder-related database operation failed
    #[error("Failed to perform reminder-related database operation: {source}")]
    Database {
        /// Underlying error type
        #[from]
        source: DieselError,
    },
}
