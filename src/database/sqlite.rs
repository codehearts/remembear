//! Database controller for persistent storage via sqlite

use super::{Database, Error};
use crate::diesel::Connection;
use diesel::sqlite::SqliteConnection;

/// Manages sqlite database connections
pub struct Sqlite {
    connection: SqliteConnection,
}

impl Database for Sqlite {
    fn connect(database_url: &str) -> Result<Self, Error> {
        let connection =
            SqliteConnection::establish(database_url).map_err(|source| Error::Connection {
                database_url: database_url.to_string(),
                source,
            })?;

        Ok(Self { connection })
    }

    fn connection(&self) -> &SqliteConnection {
        &self.connection
    }
}
