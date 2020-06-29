//! Commands for the CLI interface

mod user;

use crate::user::provider::UserManagement;
use structopt::StructOpt;

/// Providers for CLI command functionality
pub struct Providers<'a> {
    /// Provider for user functionality
    pub user: &'a dyn UserManagement,
}

/// Interface for executable CLI commands
pub trait Command {
    /// Executes the command
    ///
    /// # Errors
    ///
    /// When command execution fails, usually from a provider error
    fn execute(self, providers: Providers) -> Result<String, Box<dyn std::error::Error>>;
}

#[derive(StructOpt)]
#[structopt(name = "remembear", about = "CLI tool for recurring reminders")]
/// Global commands for the CLI interface
pub enum Global {
    /// Manage users
    User(user::User),
}

impl Command for Global {
    fn execute(self, providers: Providers) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Self::User(user_command) => user_command.execute(providers),
        }
    }
}
