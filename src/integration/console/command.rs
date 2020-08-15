use crate::integration::{model::Uid, Integration};
use crate::Providers;
use serde_json::json;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "remembear integration console")]
/// Commands for console integration management
pub enum Command {
    /// Sets the console output color for a user
    Color {
        /// UID of the user
        uid: i32,
        /// Name of the color
        color: String,
    },
    /// Removes the configuration for a user by their UID
    Remove {
        /// Uid of the user record to remove
        uid: i32,
    },
}

impl Command {
    pub fn execute<'a>(
        integration: &(dyn Integration + 'a),
        providers: &Providers,
        arguments: &[String],
    ) -> Result<String, Box<dyn std::error::Error>> {
        match Self::from_iter_safe(arguments)? {
            Self::Color { uid, color } => {
                let new_record = providers.integration.set(
                    integration,
                    Uid::User(uid),
                    json!({ "color": color }),
                )?;
                Ok(serde_json::to_string_pretty(&new_record)?)
            }
            Self::Remove { uid } => {
                match providers.integration.remove(integration, Uid::User(uid)) {
                    Ok(_) => Ok(format!(
                        "Console integration preferences removed for user {}",
                        uid
                    )),
                    Err(error) => Err(format!(
                        "Failed to remove console integration preferences for {}: {}",
                        uid, error
                    )
                    .into()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::{model::Record, Console, Error};
    use mockall::predicate::*;
    use std::io::stdout;

    #[test]
    fn it_returns_error_for_unknown_command() -> Result<(), Box<dyn std::error::Error>> {
        let console = Console(Box::new(stdout()));

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };

        let execution_result = Command::execute(
            &console,
            &providers,
            &[String::from("console"), String::from("unknown")],
        );

        assert!(execution_result.is_err());

        Ok(())
    }

    #[test]
    fn it_sets_color_for_integration() -> Result<(), Box<dyn std::error::Error>> {
        let console = Console(Box::new(stdout()));
        let mut integration_provider = crate::integration::provider::MockProvidable::new();

        integration_provider
            .expect_set()
            .with(always(), eq(Uid::User(1)), eq(json!({ "color": "red" })))
            .returning(|_, _, _| {
                Ok(Record {
                    uid: 1,
                    uid_type: "user",
                    name: "console",
                    data: String::from(r#"{"color": "red"}"#),
                })
            })
            .times(1);

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &integration_provider,
        };

        let output = Command::execute(
            &console,
            &providers,
            &[
                String::from("console"),
                String::from("color"),
                String::from("1"),
                String::from("red"),
            ],
        )?;

        let expected_output = serde_json::to_string_pretty(&Record {
            uid: 1,
            uid_type: "user",
            name: "console",
            data: String::from(r#"{"color": "red"}"#),
        })?;

        assert_eq!(expected_output, output);

        Ok(())
    }

    #[test]
    fn it_returns_error_when_set_fails() -> Result<(), Box<dyn std::error::Error>> {
        let console = Console(Box::new(stdout()));
        let mut integration_provider = crate::integration::provider::MockProvidable::new();

        integration_provider
            .expect_set()
            .returning(|_, _, _| Err(Error::JSONSerialization(String::from(""))))
            .times(1);

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &integration_provider,
        };

        let execution_result = Command::execute(
            &console,
            &providers,
            &[
                String::from("console"),
                String::from("color"),
                String::from("1"),
                String::from("red"),
            ],
        );

        assert!(execution_result.is_err());

        Ok(())
    }

    #[test]
    fn it_removes_integration_records() -> Result<(), Box<dyn std::error::Error>> {
        let console = Console(Box::new(stdout()));
        let mut integration_provider = crate::integration::provider::MockProvidable::new();

        integration_provider
            .expect_remove()
            .with(always(), eq(Uid::User(1)))
            .returning(|_, _| Ok(()))
            .times(1);

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &integration_provider,
        };

        let execution_result = Command::execute(
            &console,
            &providers,
            &[
                String::from("console"),
                String::from("remove"),
                String::from("1"),
            ],
        );

        assert!(execution_result.is_ok());

        Ok(())
    }

    #[test]
    fn it_returns_an_error_when_removal_fails() -> Result<(), Box<dyn std::error::Error>> {
        let console = Console(Box::new(stdout()));
        let mut integration_provider = crate::integration::provider::MockProvidable::new();

        integration_provider
            .expect_remove()
            .with(always(), eq(Uid::User(1)))
            .returning(|_, _| Err(Error::JSONSerialization(String::from(""))))
            .times(1);

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &integration_provider,
        };

        let execution_result = Command::execute(
            &console,
            &providers,
            &[
                String::from("console"),
                String::from("remove"),
                String::from("1"),
            ],
        );

        assert!(execution_result.is_err());

        Ok(())
    }
}
