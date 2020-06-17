//! Configuration management for loading from disk into memory

pub mod error;

use config::{Config as GenericConfig, File as ConfigFile};
use error::Error;
use serde::Deserialize;

/// All configurable properties of the app
#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    /// Configuration for the database
    pub database: Database,
}

/// All configurable database properties
#[derive(Debug, Deserialize, PartialEq)]
pub struct Database {
    /// Configuration for the sqlite database
    pub sqlite: SqliteDatabase,
}

/// All configurable sqlite database properties
#[derive(Debug, Deserialize, PartialEq)]
pub struct SqliteDatabase {
    /// Path to the sqlite database
    pub path: String,
}

impl Config {
    /// Reads the configuration into memory from remembear.yaml
    ///
    /// # Errors
    ///
    /// When remembear.yaml does not exist or is improperly formatted
    pub fn load(filename: &str) -> Result<Self, Error> {
        let mut config = GenericConfig::default();

        config
            .merge(ConfigFile::with_name(filename))
            .map_err(|source| Error::FileRead {
                filename: filename.to_string(),
                source,
            })?;

        config
            .try_into::<Config>()
            .map_err(|source| Error::InvalidSyntax {
                filename: filename.to_string(),
                source,
            })
    }
}
