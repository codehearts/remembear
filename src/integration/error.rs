//! Error types for integrations

use diesel::result::Error as DieselError;
use serde_json::Error as JSONError;
use structopt::clap::Error as ClapError;
use thiserror::Error;

/// Integration errors
#[derive(Debug, Error)]
pub enum Error {
    /// An integration-related database operation failed
    #[error("Failed to perform integration database operation: {0}")]
    Database(#[from] DieselError),
    /// A JSON serialization error occurred
    #[error("Failed to serialize JSON: {0}")]
    JSONSerialization(String),
    /// A JSON deserialization error occurred
    #[error("Failed to deserialize JSON: {0}")]
    JSONDeserialization(#[from] JSONError),
    /// A command line parsing error occurred
    #[error("Failed to parse command line arguments: {0}")]
    Command(#[from] ClapError),
}
