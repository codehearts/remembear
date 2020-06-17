//! Error types for configuration management failures

use config::ConfigError;
use thiserror::Error;

/// Configuration management errors
#[derive(Error, Debug)]
pub enum Error {
    /// The config file could not be read
    #[error("Failed to read config file {filename}: {source}")]
    FileRead {
        /// Name of the config file
        filename: String,
        /// Underlying error type
        source: ConfigError,
    },

    /// The config file contains invalid syntax
    #[error("Invalid syntax for config file {filename}: {source}")]
    InvalidSyntax {
        /// Name of the config file
        filename: String,
        /// Underlying error type
        source: ConfigError,
    },
}
