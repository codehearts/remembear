//! CLI interface commands for reminder management

use super::{Command, Providers};
use crate::reminder::model::{NewReminder, UpdatedReminder};
use crate::Schedule;
use chrono::{DateTime, Datelike, TimeZone, Utc, Weekday};
use structopt::StructOpt;

#[derive(StructOpt)]
/// Commands for reminder management
pub enum Reminder {
    /// Adds a new reminder
    Add {
        /// Name for the reminder
        name: String,
        /// Schedule for the reminder, as a JSON object of weekday to times.
        ///
        /// For example, a schedule of Monday at 10:30 and 22:30, and Wednesday at 12:30 would be:
        ///     {"mon":["10:30:00","22:30:00"],"wed":["12:30:00"]}
        schedule: String,
        /// List of assigned user uids, in order of assignment
        assignees: Vec<i32>,
    },
    /// Updates an existing reminder
    Update {
        /// Uid of the reminder to update
        uid: i32,
        /// Updated name for the reminder
        #[structopt(short, long)]
        name: Option<String>,
        /// Updated schedule for the reminder
        ///
        /// For example, a schedule of Monday at 10:30 and 22:30, and Wednesday at 12:30 would be:
        ///     {"mon":["10:30:00","22:30:00"],"wed":["12:30:00"]}
        #[structopt(short, long)]
        schedule: Option<String>,
        /// Updated list of assigned user uids, in order of assignment
        #[structopt(short, long)]
        assignees: Option<Vec<i32>>,
    },
    /// Lists all reminders as a JSON array
    List,
    /// Removes a reminder by its uid
    Remove {
        /// Uid of the reminder to remove
        uid: i32,
    },
}

impl Command for Reminder {
    fn execute(self, providers: Providers) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Self::Add {
                name,
                schedule,
                assignees,
            } => {
                let schedule = Schedule::new(
                    serde_json::from_str(&schedule)?,
                    get_start_of_this_week(),
                    assignees,
                );
                let new_reminder = providers.reminder.add(NewReminder { name, schedule })?;
                Ok(serde_json::to_string_pretty(&new_reminder)?)
            }
            Self::List => Ok(serde_json::to_string_pretty(
                &providers.reminder.get_all()?,
            )?),
            Self::Update {
                uid,
                name,
                schedule,
                assignees,
            } => match providers.reminder.get_by_uid(uid) {
                Ok(reminder) => {
                    let schedule = Schedule::new(
                        schedule.map_or(Ok(reminder.schedule.weekly_times), |schedule| {
                            serde_json::from_str(&schedule)
                        })?,
                        get_start_of_this_week(),
                        assignees.unwrap_or(reminder.schedule.assignees),
                    );

                    let updated_reminder = UpdatedReminder {
                        uid,
                        schedule,
                        name: name.unwrap_or(reminder.name),
                    };

                    let reminder = providers.reminder.update(updated_reminder)?;
                    Ok(serde_json::to_string_pretty(&reminder)?)
                }
                Err(_) => Err(format!("Invalid uid {}", uid).into()),
            },
            Self::Remove { uid } => match providers.reminder.get_by_uid(uid) {
                Ok(reminder) => {
                    providers.reminder.remove(uid)?;
                    Ok(serde_json::to_string_pretty(&reminder)?)
                }
                Err(_) => Err(format!("Invalid uid {}", uid).into()),
            },
        }
    }
}

/// Returns the start of the current week
fn get_start_of_this_week() -> DateTime<Utc> {
    let iso_week = Utc::today().iso_week();
    Utc.isoywd(iso_week.year(), iso_week.week(), Weekday::Mon)
        .and_hms(0, 0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reminder::{model, provider::MockProvidable};
    use mockall::predicate::eq;

    const SCHEDULE_ROADHOUSE: &str = r#"{"mon":["21:00:00"]}"#;
    const SCHEDULE_253: &str = r#"{"wed":["14:53:00"]}"#;
    const SCHEDULE_254: &str = r#"{"wed":["14:54:00"]}"#;

    const ASSIGNEES_ROADHOUSE: [i32; 2] = [1, 2];
    const ASSIGNEES_253: [i32; 2] = [3, 4];
    const ASSIGNEES_254: [i32; 2] = [5, 6];

    fn get_roadhouse_schedule() -> Result<Schedule, Box<dyn std::error::Error>> {
        Ok(Schedule::new(
            serde_json::from_str(SCHEDULE_ROADHOUSE)?,
            get_start_of_this_week(),
            ASSIGNEES_ROADHOUSE.to_vec(),
        ))
    }

    fn get_253_schedule() -> Result<Schedule, Box<dyn std::error::Error>> {
        Ok(Schedule::new(
            serde_json::from_str(SCHEDULE_253)?,
            get_start_of_this_week(),
            ASSIGNEES_253.to_vec(),
        ))
    }

    fn get_254_schedule() -> Result<Schedule, Box<dyn std::error::Error>> {
        Ok(Schedule::new(
            serde_json::from_str(SCHEDULE_254)?,
            get_start_of_this_week(),
            ASSIGNEES_254.to_vec(),
        ))
    }

    fn execute(
        command: Reminder,
        reminder_provider: &MockProvidable,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: reminder_provider,
        };

        command.execute(providers)
    }

    #[test]
    fn it_adds_new_reminders() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_reminder_provider = MockProvidable::new();
        let reminder = model::Reminder {
            uid: 1,
            name: String::from("Meet at Roadhouse"),
            schedule: get_roadhouse_schedule()?,
        };

        let expected_output = serde_json::to_string_pretty(&reminder)?;

        mock_reminder_provider
            .expect_add()
            .with(eq(NewReminder {
                name: String::from("Meet at Roadhouse"),
                schedule: get_roadhouse_schedule()?,
            }))
            .times(1)
            .return_once(|_| Ok(reminder));

        let output = execute(
            Reminder::Add {
                name: String::from("Meet at Roadhouse"),
                schedule: SCHEDULE_ROADHOUSE.to_string(),
                assignees: ASSIGNEES_ROADHOUSE.to_vec(),
            },
            &mock_reminder_provider,
        )?;

        assert_eq!(expected_output, output);

        Ok(())
    }

    #[test]
    fn it_lists_existing_reminders() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_reminder_provider = MockProvidable::new();
        let reminders = vec![
            model::Reminder {
                uid: 1,
                name: String::from("Meet at Roadhouse"),
                schedule: get_roadhouse_schedule()?,
            },
            model::Reminder {
                uid: 2,
                name: String::from("2:53"),
                schedule: get_253_schedule()?,
            },
        ];

        let expected_output = serde_json::to_string_pretty(&reminders)?;

        mock_reminder_provider
            .expect_get_all()
            .times(1)
            .return_once(|| Ok(reminders));

        let output = execute(Reminder::List, &mock_reminder_provider)?;

        assert_eq!(expected_output, output);

        Ok(())
    }

    #[test]
    fn it_updates_existing_reminders() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_reminder_provider = MockProvidable::new();

        let existing_reminder = model::Reminder {
            uid: 1,
            name: String::from("2:53"),
            schedule: get_253_schedule()?,
        };
        let reminder = model::Reminder {
            uid: 1,
            name: String::from("2:54"),
            schedule: get_254_schedule()?,
        };

        let expected_output = serde_json::to_string_pretty(&reminder)?;

        mock_reminder_provider
            .expect_get_by_uid()
            .with(eq(1))
            .times(1)
            .return_once(|_| Ok(existing_reminder));

        mock_reminder_provider
            .expect_update()
            .with(eq(model::UpdatedReminder {
                uid: 1,
                name: String::from("2:54"),
                schedule: get_254_schedule()?,
            }))
            .times(1)
            .return_once(|_| Ok(reminder));

        let output = execute(
            Reminder::Update {
                uid: 1,
                name: Some(String::from("2:54")),
                schedule: Some(SCHEDULE_254.to_string()),
                assignees: Some(ASSIGNEES_254.to_vec()),
            },
            &mock_reminder_provider,
        )?;

        assert_eq!(expected_output, output);

        Ok(())
    }

    #[test]
    fn it_outputs_an_error_for_invalid_update_uid() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_reminder_provider = MockProvidable::new();

        mock_reminder_provider
            .expect_get_by_uid()
            .with(eq(1))
            .times(1)
            .return_once(|_| {
                Err(crate::reminder::Error::Database {
                    source: diesel::result::Error::NotFound,
                })
            });

        let output = execute(
            Reminder::Update {
                uid: 1,
                name: Some(String::from("2:53")),
                schedule: Some(SCHEDULE_253.to_string()),
                assignees: Some(ASSIGNEES_253.to_vec()),
            },
            &mock_reminder_provider,
        );

        match output {
            Ok(_) => panic!("Error was not propagated"),
            Err(error) => assert_eq!("Invalid uid 1", error.to_string()),
        }

        Ok(())
    }

    #[test]
    fn it_removes_existing_reminders() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_reminder_provider = MockProvidable::new();

        let existing_reminder = model::Reminder {
            uid: 1,
            name: String::from("Meet at Roadhouse"),
            schedule: get_roadhouse_schedule()?,
        };

        let expected_output = serde_json::to_string_pretty(&existing_reminder)?;

        mock_reminder_provider
            .expect_get_by_uid()
            .with(eq(1))
            .times(1)
            .return_once(|_| Ok(existing_reminder));

        mock_reminder_provider
            .expect_remove()
            .with(eq(1))
            .times(1)
            .return_once(|_| Ok(()));

        let output = execute(Reminder::Remove { uid: 1 }, &mock_reminder_provider)?;

        assert_eq!(expected_output, output);

        Ok(())
    }

    #[test]
    fn it_outputs_an_error_for_invalid_remove_uid() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_reminder_provider = MockProvidable::new();

        mock_reminder_provider
            .expect_get_by_uid()
            .with(eq(1))
            .times(1)
            .return_once(|_| {
                Err(crate::reminder::Error::Database {
                    source: diesel::result::Error::NotFound,
                })
            });

        let output = execute(Reminder::Remove { uid: 1 }, &mock_reminder_provider);

        match output {
            Ok(_) => panic!("Error was not propagated"),
            Err(error) => assert_eq!("Invalid uid 1", error.to_string()),
        }

        Ok(())
    }
}
