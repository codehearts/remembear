//! Integration for displaying reminders on the console

use super::{model::Uid, Integration};
use crate::{Providers, Reminder, User};
use colored::Colorize;
use std::io::Write;
use time::{OffsetDateTime, UtcOffset};

mod command;
use command::Command;

/// Provides reminder notifications to an output buffer.
///
/// The given trait object is used as the output buffer.
/// You will almost always wants `std::io::Stdout` unless
/// you are testing the output.
pub struct Console<'a>(pub Box<dyn Write + 'a>);

impl<'a> Integration for Console<'a> {
    fn name(&self) -> &'static str {
        "console"
    }

    fn execute(
        &self,
        providers: Providers,
        arguments: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Command::execute(self, &providers, &arguments)
    }

    fn notify(
        &mut self,
        providers: &Providers,
        reminder: &Reminder,
        assignees: &[User],
        timestamp: &OffsetDateTime,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert the UTC timestamp to the local timezone
        let local_timestamp = format_date(
            timestamp.to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC)),
        );

        // Concatenate the assignees' names for human-readable display
        let assignee_names = assignees
            .iter()
            .map(|assignee| {
                let user_configuration = providers
                    .integration
                    .get(self, Uid::User(assignee.uid))
                    .unwrap_or_default();

                let name = assignee.name.clone();

                // Apply color to the name if one is set for the user
                match user_configuration.get("color") {
                    Some(color) => match color.as_str() {
                        Some(color) => name.color(color).to_string(),
                        None => name,
                    },
                    None => name,
                }
            })
            .collect::<Vec<String>>()
            .join(", ");

        // Write to the output buffer
        self.0.write_fmt(format_args!(
            "[{}] {}: {}",
            local_timestamp, reminder.name, assignee_names,
        ))?;

        Ok(())
    }
}

/// Formats a date for human-readable output
fn format_date(datetime: OffsetDateTime) -> String {
    let timezone = datetime.offset();
    let (year, month, day) = datetime.to_calendar_date();
    let (hour, minute, second) = datetime.to_hms();

    format!(
        "{}-{:0.2}-{:0.2} {:0.2}:{:0.2}:{:0.2} {:+.3}",
        year,
        u8::from(month),
        day,
        hour,
        minute,
        second,
        timezone.whole_hours()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::Error;
    use crate::schedule::model::WeeklyTimes;
    use crate::{Reminder, Schedule, User};
    use mockall::predicate::*;
    use std::io::stdout;
    use time::macros::datetime;

    #[test]
    fn it_has_proper_name() {
        let console = Console(Box::new(stdout()));
        assert_eq!("console", console.name());
    }

    #[test]
    fn it_outputs_uncolored_names_on_notify() -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = datetime!(2020-01-01 00:01:02 UTC);
        let local_timestamp = format_date(
            timestamp.to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC)),
        );

        let config_1 = Ok(serde_json::json!({}));
        let config_2 = Ok(serde_json::json!({}));

        // Expect the UTC timestamp to be in the correct timezone
        let expected_output = format!("[{}] Reminder: Laura, Donna", local_timestamp);
        let actual_output = get_console_output(timestamp, config_1, config_2)?;

        assert_eq!(expected_output, actual_output);

        Ok(())
    }

    #[test]
    fn it_outputs_colored_names_on_notify() -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = datetime!(2020-01-01 00:01:02 UTC);
        let local_timestamp = format_date(
            timestamp.to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC)),
        );

        let assignees = format!("{}, {}", "Laura".color("red"), "Donna".color("green"));

        let config_1 = Ok(serde_json::json!({"color": "red"}));
        let config_2 = Ok(serde_json::json!({"color": "green"}));

        // Expect the UTC timestamp to be in the correct timezone
        let expected_output = format!("[{}] Reminder: {}", local_timestamp, assignees);
        let actual_output = get_console_output(timestamp, config_1, config_2)?;

        assert_eq!(expected_output, actual_output);

        Ok(())
    }

    #[test]
    fn it_outputs_despite_provider_failures_on_notify() -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = datetime!(2020-01-01 00:01:02 UTC);
        let local_timestamp = format_date(
            timestamp.to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC)),
        );

        let config_1 = Err(Error::JSONSerialization(String::from("dummy error")));
        let config_2 = Err(Error::JSONSerialization(String::from("dummy error")));

        // Expect the UTC timestamp to be in the correct timezone
        let expected_output = format!("[{}] Reminder: Laura, Donna", local_timestamp);
        let actual_output = get_console_output(timestamp, config_1, config_2)?;

        assert_eq!(expected_output, actual_output);

        Ok(())
    }

    /// Runs the console integration and returns the output to its buffer.
    /// There will be 2 users for this integration:
    ///
    /// 1. Laura
    /// 2. Donna
    ///
    /// There is one reminder set called "Reminder" which all users are assigned to.
    ///
    /// # Errors
    ///
    /// When the console integration fails to run or the output is not UTF-8.
    fn get_console_output(
        timestamp: OffsetDateTime,
        config_1: Result<serde_json::Value, Error>,
        config_2: Result<serde_json::Value, Error>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut integration_provider = crate::integration::provider::MockProvidable::new();

        integration_provider
            .expect_get()
            .with(always(), eq(Uid::User(1)))
            .return_once(|_, _| config_1)
            .times(1);

        integration_provider
            .expect_get()
            .with(always(), eq(Uid::User(2)))
            .return_once(|_, _| config_2)
            .times(1);

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &integration_provider,
        };

        let reminder = Reminder {
            uid: 1,
            name: String::from("Reminder"),
            schedule: Schedule::new(WeeklyTimes::default(), timestamp, vec![1, 2]),
        };

        let assignees = vec![
            User {
                uid: 1,
                name: String::from("Laura"),
            },
            User {
                uid: 2,
                name: String::from("Donna"),
            },
        ];

        let mut output_buffer = Vec::<u8>::new();

        {
            let mut integration = Console(Box::new(&mut output_buffer));
            integration.notify(&providers, &reminder, &assignees, &timestamp)?;
        }

        Ok(String::from_utf8(output_buffer)?)
    }
}
