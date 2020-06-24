//! Integration tests for managing persistent storage via a database

use remembear::database::{self, error::Error, Database};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Creates a temporary sqlite3 file on disk, returning the path
fn get_temp_database_url() -> Result<String> {
    let temp_file = tempfile::Builder::new().suffix(".sqlite3").tempfile()?;
    Ok(temp_file.path().display().to_string())
}

#[test]
fn it_creates_sqlite_database_object_when_connection_succeeds() -> Result<()> {
    let database_url = get_temp_database_url()?;
    database::Sqlite::connect(&database_url)?;

    Ok(())
}

#[test]
fn it_returns_connection_error_for_bad_sqlite_database_url() -> Result<()> {
    let invalid_database_url = "localhost/bad_url";

    match database::Sqlite::connect(&invalid_database_url) {
        Err(Error::Connection { database_url, .. }) => {
            assert_eq!(invalid_database_url, database_url)
        }
        Err(error) => panic!("Connection wasn't returned, got {}", error),
        Ok(_) => panic!("Invalid database url successfully connected"),
    }

    Ok(())
}
