//! Models and functionality for remembear integrations

pub mod model;
pub mod provider;

pub use console::Console;
pub use error::Error;
pub use provider::Provider;

mod console;
mod error;

use crate::{config, Config, Providers, Reminder, User};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

#[cfg(test)]
use mockall::automock;

/// Provides an interface for integration with external services
#[cfg_attr(test, automock)]
pub trait Integration {
    /// Providers the unique name of the integration
    fn name(&self) -> &'static str;

    /// Executes a set of command-line arguments for the integration
    ///
    /// # Errors
    ///
    /// When command-line argument parsing fails
    #[allow(clippy::needless_lifetimes)] // 'a is needed for mockall test target
    fn execute<'a>(
        &self,
        providers: Providers<'a>,
        arguments: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Notifies the integrated service of a triggered reminder
    ///
    /// # Errors
    ///
    /// If integration with the external service fails
    fn notify<'a>(
        &mut self,
        providers: &Providers<'a>,
        reminder: &Reminder,
        assignees: &[User],
        timestamp: &DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

/// All configured integrations for the service
#[derive(Default)]
pub struct Integrations(BTreeMap<&'static str, Box<dyn Integration>>);

impl Integrations {
    /// Initializes all configured integrations
    ///
    /// # Errors
    ///
    /// When there is an error with the config or any of the integrations
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let mut integrations: BTreeMap<&'static str, Box<dyn Integration>> = BTreeMap::new();

        if let Some(configs) = &config.integrations {
            if Self::get_enabled_config(configs, "console").is_some() {
                let integration = Console(Box::new(std::io::stdout()));
                integrations.insert(integration.name(), Box::new(integration));
            }
        }

        Self(integrations)
    }

    /// Returns an integration configuration if the integration is enabled
    fn get_enabled_config<'a>(
        config: &'a config::Integrations,
        integration_name: &str,
    ) -> Option<&'a config::Integration> {
        config.get(integration_name).filter(|integration_config| {
            integration_config.get("enabled") == Some(&String::from("true"))
        })
    }
}

impl Deref for Integrations {
    type Target = BTreeMap<&'static str, Box<dyn Integration>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Integrations {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
