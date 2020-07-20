//! Commands for the CLI interface

mod reminder;
mod user;

use crate::Scheduler;
use structopt::StructOpt;

/// Providers for CLI command functionality
pub struct Providers<'a> {
    /// Provider for user functionality
    pub user: &'a dyn crate::user::provider::Providable,
    /// Provider for reminder functionality
    pub reminder: &'a dyn crate::reminder::provider::Providable,
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
    /// Manage reminders
    Reminder(reminder::Reminder),
    /// Start the scheduler
    Start,
}

impl Command for Global {
    fn execute(self, providers: Providers) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Self::User(command) => command.execute(providers),
            Self::Reminder(command) => command.execute(providers),
            Self::Start => Ok(String::from("")),
        }
    }
}

/// Executes the given global command
///
/// # Errors
///
/// If the scheduler is started and a reminder is triggered when the queue is empty
pub async fn execute(
    command: Global,
    providers: Providers<'_>,
) -> Result<String, Box<dyn std::error::Error>> {
    match command {
        Global::Start => {
            let mut scheduler = Scheduler::new(providers.reminder.get_all()?);
            scheduler.run().await?;
            Ok(String::from("Scheduler queue is empty"))
        }
        _ => command.execute(providers),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_does_nothing_when_executing_start_synchronously() -> Result<(), Box<dyn std::error::Error>>
    {
        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
        };
        assert_eq!(String::from(""), Global::Start.execute(providers)?);

        Ok(())
    }
}
