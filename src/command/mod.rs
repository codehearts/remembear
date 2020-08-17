//! Commands for the CLI interface

mod reminder;
mod user;

use crate::{Integrations, Providers, Scheduler};
use structopt::StructOpt;

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
    /// Manage integrations
    #[structopt(external_subcommand)]
    Integration(Vec<String>),
    /// Start the scheduler
    Start,
}

impl Command for Global {
    fn execute(self, providers: Providers) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Self::User(command) => command.execute(providers),
            Self::Reminder(command) => command.execute(providers),
            // These commands are handled by the async `execute` function
            Self::Start | Self::Integration(_) => Ok(String::from("")),
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
    integrations: Integrations,
) -> Result<String, Box<dyn std::error::Error>> {
    match command {
        // Start the scheduler if requested
        Global::Start => {
            let mut scheduler =
                Scheduler::new(providers.reminder.get_all()?, providers, integrations);
            scheduler.run().await?;
            Ok(String::from("Scheduler queue is empty"))
        }
        // Find the specific integration if possible, and pass execution on to it
        Global::Integration(mut arguments) => {
            arguments.remove(0); // Remove "integration"
            let integration_name = arguments.remove(0);
            match integrations.get(integration_name.as_str()) {
                Some(integration) => integration.execute(providers, arguments),
                None => Err(format!("Invalid integration `{}`", integration_name).into()),
            }
        }
        _ => command.execute(providers),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[test]
    fn it_does_nothing_when_executing_start_synchronously() -> Result<(), Box<dyn std::error::Error>>
    {
        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };
        assert_eq!(String::from(""), Global::Start.execute(providers)?);

        Ok(())
    }

    #[test]
    fn it_does_nothing_when_executing_integration_synchronously(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };
        assert_eq!(
            String::from(""),
            Global::Integration(vec![]).execute(providers)?
        );

        Ok(())
    }

    #[tokio::test]
    async fn it_fails_when_running_unknown_integration() -> Result<(), Box<dyn std::error::Error>> {
        let config: Config = serde_json::from_str(
            r#"{
            "database": {
                "sqlite": {
                    "path": "remembear.sqlite3"
                }
            },
            "integrations": {
                "known": {
                    "enabled": "true"
                    }
            }
        }"#,
        )?;

        let command =
            Global::Integration(vec![String::from("integration"), String::from("unknown")]);

        let integrations = Integrations::new(&config);

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };

        assert!(execute(command, providers, integrations).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn it_runs_known_integrations() -> Result<(), Box<dyn std::error::Error>>
    {
        let config: Config = serde_json::from_str(r#"{
            "database": {
                "sqlite": {
                    "path": "remembear.sqlite3"
                }
            },
            "integrations": {
                "known": {
                    "enabled": "true"
                }
            }
        }"#)?;

        let command = Global::Integration(vec![String::from("integration"), String::from("known")]);

        let integrations = Integrations::new(&config);

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };

        assert!(execute(command, providers, integrations).await.is_err());

        Ok(())
    }
}
