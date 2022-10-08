//! Data model for a new reminder

use crate::database::schema::reminders;
use crate::{schedule, Schedule};

/// Necessary data to create a new reminder
#[derive(Debug, Eq, PartialEq)]
pub struct NewReminder {
    /// Name of the reminder
    pub name: String,
    /// Schedule for the reminder
    pub schedule: Schedule,
}

/// Insertable `NewReminder` for use with `diesel`
#[derive(Debug, Insertable, Eq, PartialEq)]
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
    use time::{Date, Weekday};

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn it_converts_into_insertable_new_reminder() -> Result<()> {
        let new_reminder = NewReminder {
            name: String::from("Meet at Roadhouse"),
            schedule: Schedule::new(
                vec![].into_iter().collect(),
                Date::from_iso_week_date(1989, 13, Weekday::Monday)?
                    .midnight()
                    .assume_utc(),
                vec![],
            ),
        };

        let expected_new_reminder = InsertableNewReminder {
            name: String::from("Meet at Roadhouse"),
            schedule: Schedule::new(
                vec![].into_iter().collect(),
                Date::from_iso_week_date(1989, 13, Weekday::Monday)?
                    .midnight()
                    .assume_utc(),
                vec![],
            )
            .into(),
        };

        assert_eq!(expected_new_reminder, new_reminder.into());

        Ok(())
    }
}
