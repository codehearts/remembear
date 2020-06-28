//! Integration tests for managing persistent storage via a database

mod common;

use common::Result;
use remembear::database::{self, error::Error, Database};

#[test]
fn it_creates_sqlite_database_object_when_connection_succeeds() -> Result<()> {
    database::Sqlite::connect("../remembear.sqlite3")?;
    Ok(())
}

#[test]
fn it_returns_connection_error_for_bad_sqlite_database_url() -> Result<()> {
    let invalid_database_url = "localhost/bad_url";

    match database::Sqlite::connect(&invalid_database_url) {
        Err(Error::Connection { database_url, .. }) => {
            assert_eq!(invalid_database_url, database_url)
        }
        Ok(_) => panic!("Invalid database url successfully connected"),
    }

    Ok(())
}
