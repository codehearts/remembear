//! Error types for database operation failures

use diesel::result::ConnectionError;
use diesel::result::Error as DieselError;
use thiserror::Error;

/// Database operation errors
#[derive(Error, Debug)]
pub enum Error {
    /// The database connection failed
    #[error("Failed to connect to {database_url}: {source}")]
    Connection {
        /// Database url which failed to connect
        database_url: String,
        /// Underlying error type
        source: ConnectionError,
    },
    /// A database insertion operation failed
    #[error("Failed to insert into database: {source}")]
    Insertion {
        /// Underlying error type
        #[from]
        source: DieselError,
    },
}
