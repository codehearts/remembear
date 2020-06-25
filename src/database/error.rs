//! Error types for database operation failures

use diesel::result::ConnectionError;
use thiserror::Error;

/// Database operation errors
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// The database connection failed
    #[error("Failed to connect to {database_url}: {source}")]
    Connection {
        /// Database url which failed to connect
        database_url: String,
        /// Underlying error type
        source: ConnectionError,
    },
}
