//! 🐻 Self-hosted web app for recurring reminders
//!
//! Remembear was created to manage household chores but can be used for medication reminders,
//! appointment notifications, and anything else occuring on a regular weekly or daily basis!

#![deny(clippy::all, clippy::pedantic, missing_docs, warnings)]

#[macro_use]
extern crate diesel;

pub mod command;
pub mod config;
pub mod database;
pub mod reminder;
pub mod schedule;
pub mod scheduler;
pub mod user;

pub use reminder::model::Reminder;
pub use schedule::model::Schedule;
pub use scheduler::model::Scheduler;
pub use user::model::User;

use crate::config::Config;
use database::Database;
use std::error::Error;
use std::sync::Arc;

/// All dependencies for the service
pub struct Dependencies {
    /// Database connection for modules needing persistent storage
    pub database: Arc<dyn database::Database>,
}

/// Initializes and configures all dependencies
///
/// # Errors
///
/// When there is an error with the config or any of the dependencies
pub fn initialize_dependencies() -> Result<Dependencies, Box<dyn Error>> {
    let config = Config::load("remembear")?;
    let database = Arc::new(database::Sqlite::connect(&config.database.sqlite.path)?);

    Ok(Dependencies { database })
}
