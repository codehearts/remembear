//! Shared command line interface functionality between integration tests

use super::common::Result;
use super::common_database;
use remembear::{command, command::Command, reminder, user};
use structopt::StructOpt;

/// Provides a simple interface for executing CLI commands
pub struct Executor {
    user: user::Provider,
    reminder: reminder::Provider,
}

impl Executor {
    /// Creates a new executor with its own providers
    pub fn new() -> Result<Self> {
        let database = common_database::new()?;

        Ok(Self {
            user: user::Provider::new(database.clone()),
            reminder: reminder::Provider::new(database.clone()),
        })
    }

    /// Executes a CLI command
    pub fn execute(&self, command: &[&str]) -> Result<String> {
        command::Global::from_iter(command).execute(command::Providers {
            user: &self.user,
            reminder: &self.reminder,
        })
    }
}
