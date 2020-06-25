//! Database controller for persistent storage

pub mod error;
pub mod schema;
mod sqlite;

pub use error::Error;
pub use sqlite::Sqlite;

/// Interface to manage a database connection
pub trait Database {
    /// Connects to the provided database url
    ///
    /// # Errors
    ///
    /// When a connection to the database can not be established
    fn connect(database_url: &str) -> Result<Self, Error>
    where
        Self: Sized;

    /// Provides a reference to the established database connection
    fn connection(&self) -> &diesel::sqlite::SqliteConnection;
}
