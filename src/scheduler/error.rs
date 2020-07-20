//! Error types for a real-time scheduler

use thiserror::Error;

/// Real-time scheduler errors
#[derive(Debug, Error)]
pub enum Error {
    /// A scheduled entity would have been processed if the queue wasn't empty
    #[error("A reminder was scheduled but nothing is in the scheduler queue")]
    QueueEmpty(#[from] tokio::time::Error),
}
