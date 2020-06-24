//! Database controller for persistent storage

pub mod error;
pub mod schema;
mod sqlite;

#[cfg(test)]
use mockall::automock;

use diesel::insertable::CanInsertInSingleQuery;
use diesel::query_builder::QueryFragment;
use diesel::sqlite::Sqlite as SqliteBackend;

pub use error::Error;
pub use sqlite::Sqlite;

/// Interface to manage a database connection
#[cfg_attr(test, automock)]
pub trait Database {
    /// Connects to the provided database url
    ///
    /// # Errors
    ///
    /// When a connection to the database can not be established
    fn connect(database_url: &str) -> Result<Self, Error>
    where
        Self: Sized;

}
