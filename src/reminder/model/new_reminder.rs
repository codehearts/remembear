//! Data model for a new reminder

use crate::database::schema::reminders;
use crate::{schedule, Schedule};

/// Necessary data to create a new reminder
pub struct NewReminder {
    /// Name of the reminder
    pub name: String,
    /// Schedule for the reminder
    pub schedule: Schedule,
}

/// Insertable `NewReminder` for use with `diesel`
#[derive(Debug, Insertable, PartialEq)]
#[table_name = "reminders"]
pub(crate) struct InsertableNewReminder {
    /// Name of the reminder
    pub name: String,
    /// Schedule for the reminder
    #[diesel(embed)]
    pub schedule: schedule::Provider,
}

impl From<NewReminder> for InsertableNewReminder {
    fn from(new_reminder: NewReminder) -> Self {
        Self {
            name: new_reminder.name,
            schedule: new_reminder.schedule.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc, Weekday};

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn it_converts_into_insertable_new_reminder() -> Result<()> {
        let new_reminder = NewReminder {
            name: String::from("Meet at Roadhouse"),
            schedule: Schedule::new(
                vec![].into_iter().collect(),
                Utc.isoywd(1989, 13, Weekday::Mon).and_hms(0, 0, 0),
                vec![],
            ),
        };

        let expected_new_reminder = InsertableNewReminder {
            name: String::from("Meet at Roadhouse"),
            schedule: Schedule::new(
                vec![].into_iter().collect(),
                Utc.isoywd(1989, 13, Weekday::Mon).and_hms(0, 0, 0),
                vec![],
            )
            .into(),
        };

        assert_eq!(expected_new_reminder, new_reminder.into());

        Ok(())
    }
}
