//! Database controller for persistent storage

pub mod error;
mod schema;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use error::Error;

/// Manages database connections
pub struct Database {
    _connection: SqliteConnection,
}

impl Database {
    /// Connects to the provided database url
    ///
    /// # Errors
    ///
    /// When a connection to the database can not be established
    pub fn connect(database_url: &str) -> Result<Self, Error> {
        let connection =
            SqliteConnection::establish(database_url).map_err(|source| Error::Connection {
                database_url: database_url.to_string(),
                source,
            })?;

        Ok(Self {
            _connection: connection,
        })
    }
}
