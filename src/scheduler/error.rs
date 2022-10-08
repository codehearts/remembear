//! Error types for a real-time scheduler

use thiserror::Error;

/// Real-time scheduler errors
#[derive(Debug, Error)]
pub enum Error {
    /// A scheduled entity would have been processed if the queue wasn't empty
    #[error("A reminder was scheduled but nothing is in the scheduler queue")]
    QueueEmpty(#[from] tokio::time::error::Error),
    /// A scheduled entity is unavailable for notification
    #[error("The scheduled reminder {0} could not be obtained from the queue")]
    Unavailable(i32),
    /// Assignees are unavailable for the scheduled reminder
    #[error("Assignees could not be obtained for the scheduled reminder")]
    Assignees(#[from] crate::user::Error),
    /// An integration failed to notify of a scheduled reminder
    #[error("Integration failed to notify of a scheduled reminder")]
    Integration(#[from] Box<dyn std::error::Error>),
}
