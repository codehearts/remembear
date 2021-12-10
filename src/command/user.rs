//! CLI interface commands for user management

use super::{Command, Providers};
use crate::user::model::{NewUser, UpdatedUser};
use structopt::StructOpt;

#[derive(StructOpt)]
/// Commands for user management
pub enum User {
    /// Adds a new user
    Add {
        /// User name
        name: String,
    },
    /// Updates an existing user
    Update {
        /// Uid of the user to update
        uid: i32,
        /// Updated name for the user
        #[structopt(short, long)]
        name: Option<String>,
    },
    /// Lists all users as a JSON array
    List,
    /// Removes a user by their uid
    Remove {
        /// Uid of the user to remove
        uid: i32,
    },
}

impl Command for User {
    fn execute(self, providers: Providers) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Self::Add { name } => {
                let new_user = providers.user.add(NewUser { name })?;
                Ok(serde_json::to_string_pretty(&new_user)?)
            }
            Self::List => Ok(serde_json::to_string_pretty(&providers.user.get_all()?)?),
            Self::Update { uid, name } => match providers.user.get_by_uid(uid) {
                Ok(user) => {
                    let mut updated_user: UpdatedUser = user;

                    if let Some(name) = name {
                        updated_user.name = name;
                    }

                    let user = providers.user.update(updated_user)?;
                    Ok(serde_json::to_string_pretty(&user)?)
                }
                Err(_) => Err(format!("Invalid uid {}", uid).into()),
            },
            Self::Remove { uid } => match providers.user.get_by_uid(uid) {
                Ok(user) => {
                    providers.user.remove(uid)?;
                    Ok(serde_json::to_string_pretty(&user)?)
                }
                Err(_) => Err(format!("Invalid uid {}", uid).into()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::{self, model, provider::MockProvidable};
    use mockall::predicate::eq;

    fn execute(
        command: User,
        user_provider: &MockProvidable,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let providers = Providers {
            user: user_provider,
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };

        command.execute(providers)
    }

    #[test]
    fn it_adds_new_users() -> Result<(), Box<dyn std::error::Error>> {
        let name = String::from("Leland");

        let mut mock_user_provider = MockProvidable::new();
        let user = model::User {
            uid: 1,
            name: name.clone(),
        };

        let expected_output = serde_json::to_string_pretty(&user)?;

        mock_user_provider
            .expect_add()
            .with(eq(NewUser { name: name.clone() }))
            .times(1)
            .return_once(|_| Ok(user));

        let output = execute(User::Add { name }, &mock_user_provider)?;

        assert_eq!(expected_output, output);

        Ok(())
    }

    #[test]
    fn it_lists_existing_users() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_user_provider = MockProvidable::new();
        let users = vec![
            model::User {
                uid: 1,
                name: String::from("Leland"),
            },
            model::User {
                uid: 2,
                name: String::from("Sarah"),
            },
        ];

        let expected_output = serde_json::to_string_pretty(&users)?;

        mock_user_provider
            .expect_get_all()
            .times(1)
            .return_once(|| Ok(users));

        let output = execute(User::List, &mock_user_provider)?;

        assert_eq!(expected_output, output);

        Ok(())
    }

    #[test]
    fn it_updates_existing_users() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_user_provider = MockProvidable::new();

        let existing_user = model::User {
            uid: 1,
            name: String::from("Leland"),
        };
        let updated_user = model::UpdatedUser {
            uid: 1,
            name: String::from("Bob"),
        };
        let user = model::User {
            uid: 1,
            name: String::from("Bob"),
        };

        let expected_output = serde_json::to_string_pretty(&user)?;

        mock_user_provider
            .expect_get_by_uid()
            .with(eq(1))
            .times(1)
            .return_once(|_| Ok(existing_user));

        mock_user_provider
            .expect_update()
            .with(eq(updated_user))
            .times(1)
            .return_once(|_| Ok(user));

        let output = execute(
            User::Update {
                uid: 1,
                name: Some(String::from("Bob")),
            },
            &mock_user_provider,
        )?;

        assert_eq!(expected_output, output);

        Ok(())
    }

    #[test]
    fn it_outputs_an_error_for_invalid_update_uid() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_user_provider = MockProvidable::new();

        mock_user_provider
            .expect_get_by_uid()
            .with(eq(1))
            .times(1)
            .return_once(|_| {
                Err(user::Error::Database {
                    source: diesel::result::Error::NotFound,
                })
            });

        let output = execute(
            User::Update {
                uid: 1,
                name: Some(String::from("Bob")),
            },
            &mock_user_provider,
        );

        match output {
            Ok(_) => panic!("Error was not propagated"),
            Err(error) => assert_eq!("Invalid uid 1", error.to_string()),
        }

        Ok(())
    }

    #[test]
    fn it_removes_existing_users() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_user_provider = MockProvidable::new();

        let existing_user = model::User {
            uid: 1,
            name: String::from("Leland"),
        };

        let expected_output = serde_json::to_string_pretty(&existing_user)?;

        mock_user_provider
            .expect_get_by_uid()
            .with(eq(1))
            .times(1)
            .return_once(|_| Ok(existing_user));

        mock_user_provider
            .expect_remove()
            .with(eq(1))
            .times(1)
            .return_once(|_| Ok(()));

        let output = execute(User::Remove { uid: 1 }, &mock_user_provider)?;

        assert_eq!(expected_output, output);

        Ok(())
    }

    #[test]
    fn it_outputs_an_error_for_invalid_remove_uid() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_user_provider = MockProvidable::new();

        mock_user_provider
            .expect_get_by_uid()
            .with(eq(1))
            .times(1)
            .return_once(|_| {
                Err(user::Error::Database {
                    source: diesel::result::Error::NotFound,
                })
            });

        let output = execute(User::Remove { uid: 1 }, &mock_user_provider);

        match output {
            Ok(_) => panic!("Error was not propagated"),
            Err(error) => assert_eq!("Invalid uid 1", error.to_string()),
        }

        Ok(())
    }
}
