//! Provider for stored scheduling data

mod model;

use super::{model::Schedule, Error};
use crate::database::schema::reminders;
use diesel::deserialize::{FromSqlRow, Result as FromSqlResult};
use diesel::sql_types::{Integer, Text};
use diesel::{backend::Backend, row::Row};
use model::{StoredAssignees, StoredIsoWeek, StoredWeeklyTimes};
use serde::Deserialize;
use std::convert::{From, TryFrom, TryInto};

/// Provides access to scheduling data in persistent storage
#[derive(Debug, Deserialize, Eq, Insertable, PartialEq, Queryable)]
#[table_name = "reminders"]
pub struct Provider {
    /// Scheduled times of day throughout the week
    #[column_name = "schedule"]
    pub(crate) weekly_times: StoredWeeklyTimes,
    /// Week in which the schedule started
    pub(crate) start_week: StoredIsoWeek,
    /// Assignee ids in order of assignment
    pub(crate) assignees: StoredAssignees,
}

impl<TDatabase> FromSqlRow<(Text, Integer, Text), TDatabase> for Provider
where
    TDatabase: Backend,
    StoredWeeklyTimes: FromSqlRow<Text, TDatabase>,
    StoredIsoWeek: FromSqlRow<Integer, TDatabase>,
    StoredAssignees: FromSqlRow<Text, TDatabase>,
{
    const FIELDS_NEEDED: usize = 3;

    fn build_from_row<TRow: Row<TDatabase>>(row: &mut TRow) -> FromSqlResult<Self> {
        Ok(Self {
            weekly_times: StoredWeeklyTimes::build_from_row(row)?,
            start_week: StoredIsoWeek::build_from_row(row)?,
            assignees: StoredAssignees::build_from_row(row)?,
        })
    }
}

impl From<Schedule> for Provider {
    fn from(schedule: Schedule) -> Self {
        let iso_week = schedule.start_date.iso_week();

        Self {
            weekly_times: StoredWeeklyTimes(schedule.weekly_times),
            start_week: StoredIsoWeek {
                year: schedule.start_date.year(),
                week: i32::try_from(iso_week.max(1).min(53)).unwrap_or(1),
            },
            assignees: StoredAssignees(schedule.assignees),
        }
    }
}

impl TryInto<Schedule> for Provider {
    type Error = Error;

    fn try_into(self) -> Result<Schedule, Self::Error> {
        Ok(Schedule::new(
            self.weekly_times.0,
            self.start_week.try_into()?,
            self.assignees.0,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{macros::time, Date, Weekday};

    #[test]
    fn it_converts_from_schedule() -> Result<(), Box<dyn std::error::Error>> {
        let schedule = Schedule::new(
            vec![(Weekday::Monday, vec![time!(10:30:00), time!(22:30:00)])]
                .into_iter()
                .collect(),
            Date::from_iso_week_date(2020, 2, Weekday::Monday)?
                .midnight()
                .assume_utc(),
            vec![1, 2, 3],
        );

        let expected_provider = Provider {
            weekly_times: StoredWeeklyTimes(
                vec![(Weekday::Monday, vec![time!(10:30:00), time!(22:30:00)])]
                    .into_iter()
                    .collect(),
            ),
            start_week: StoredIsoWeek {
                week: 2,
                year: 2020,
            },
            assignees: StoredAssignees(vec![1, 2, 3]),
        };

        assert_eq!(expected_provider, schedule.into());
        Ok(())
    }

    #[test]
    fn it_converts_to_schedule() -> Result<(), Box<dyn std::error::Error>> {
        let provider = Provider {
            weekly_times: StoredWeeklyTimes(
                vec![(Weekday::Monday, vec![time!(10:30:00), time!(22:30:00)])]
                    .into_iter()
                    .collect(),
            ),
            start_week: StoredIsoWeek {
                week: 2,
                year: 2020,
            },
            assignees: StoredAssignees(vec![1, 2, 3]),
        };

        let expected_schedule = Schedule::new(
            vec![(Weekday::Monday, vec![time!(10:30:00), time!(22:30:00)])]
                .into_iter()
                .collect(),
            Date::from_iso_week_date(2020, 2, Weekday::Monday)?
                .midnight()
                .assume_utc(),
            vec![1, 2, 3],
        );

        assert_eq!(Ok(expected_schedule), provider.try_into());
        Ok(())
    }

    #[test]
    fn it_fails_to_convert_to_schedule_with_invalid_iso_week(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let provider = Provider {
            weekly_times: StoredWeeklyTimes(
                vec![(Weekday::Monday, vec![time!(10:30:00), time!(22:30:00)])]
                    .into_iter()
                    .collect(),
            ),
            start_week: StoredIsoWeek {
                week: 256,
                year: 2020,
            },
            assignees: StoredAssignees(vec![1, 2, 3]),
        };

        let expected_error: Result<Schedule, _> = Err(Error::WeekTooLarge(256));

        assert_eq!(expected_error, provider.try_into());
        Ok(())
    }
}
