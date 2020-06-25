//! Error types for user operations

use diesel::result::Error as DieselError;
use thiserror::Error;

/// User operation errors
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// A user-related database operation failed
    #[error("Failed to perform user-related database operation: {source}")]
    Database {
        /// Underlying error type
        #[from]
        source: DieselError,
    },
}
