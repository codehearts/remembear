//! Shared functionality between integration tests

/// `Result` type for integration tests
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
