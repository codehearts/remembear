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
pub mod integration;
pub mod reminder;
pub mod schedule;
pub mod scheduler;
pub mod user;

pub use crate::config::Config;
pub use command::execute;
pub use integration::{Integration, Integrations};
pub use reminder::model::Reminder;
pub use schedule::model::Schedule;
pub use scheduler::model::Scheduler;
pub use user::model::User;

use database::Database;
use std::error::Error;
use std::sync::Arc;

/// All dependencies for the service
pub struct Dependencies {
    /// Database connection for modules needing persistent storage
    pub database: Arc<dyn database::Database>,
}

impl Dependencies {
    /// Initializes and configures all dependencies
    ///
    /// # Errors
    ///
    /// When there is an error with the config or any of the dependencies
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let database = Arc::new(database::Sqlite::connect(&config.database.sqlite.path)?);

        Ok(Self { database })
    }
}

/// Providers for service data
pub struct Providers<'a> {
    /// Provider for user data
    pub user: &'a dyn crate::user::provider::Providable,
    /// Provider for reminder data
    pub reminder: &'a dyn crate::reminder::provider::Providable,
}
