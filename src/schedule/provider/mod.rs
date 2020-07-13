//! Provider for stored scheduling data

mod model;

use super::{model::Schedule, Error};
use crate::database::schema::reminders;
use diesel::deserialize::{FromSqlRow, Result as FromSqlResult};
use diesel::sql_types::{Integer, Text};
use diesel::{backend::Backend, row::Row};
use model::{StoredAssignees, StoredIsoWeek, StoredWeeklyTimes};
use serde::Deserialize;
use std::convert::TryInto;

/// Provides access to scheduling data in persistent storage
#[derive(Debug, Deserialize, Insertable, PartialEq, Queryable)]
#[table_name = "reminders"]
pub struct Provider {
    /// Scheduled times of day throughout the week
    #[column_name = "schedule"]
    weekly_times: StoredWeeklyTimes,
    /// Week in which the schedule started
    start_week: StoredIsoWeek,
    /// Assignee ids in order of assignment
    assignees: StoredAssignees,
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
    use chrono::{offset::TimeZone, NaiveTime, Utc, Weekday};

    #[test]
    fn it_converts_to_schedule() {
        let provider = Provider {
            weekly_times: StoredWeeklyTimes(
                vec![(
                    Weekday::Mon,
                    vec![
                        NaiveTime::from_hms(10, 30, 0),
                        NaiveTime::from_hms(22, 30, 0),
                    ],
                )]
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
            vec![(
                Weekday::Mon,
                vec![
                    NaiveTime::from_hms(10, 30, 0),
                    NaiveTime::from_hms(22, 30, 0),
                ],
            )]
            .into_iter()
            .collect(),
            Utc.isoywd(2020, 2, Weekday::Mon).and_hms(0, 0, 0),
            vec![1, 2, 3],
        );

        assert_eq!(Ok(expected_schedule), provider.try_into());
    }

    #[test]
    fn it_fails_to_convert_to_schedule_with_invalid_iso_week() {
        let provider = Provider {
            weekly_times: StoredWeeklyTimes(
                vec![(
                    Weekday::Mon,
                    vec![
                        NaiveTime::from_hms(10, 30, 0),
                        NaiveTime::from_hms(22, 30, 0),
                    ],
                )]
                .into_iter()
                .collect(),
            ),
            start_week: StoredIsoWeek {
                week: 256,
                year: 2020,
            },
            assignees: StoredAssignees(vec![1, 2, 3]),
        };

        let expected_error: Result<Schedule, _> = Err(Error::InvalidStartWeek {
            week: 256,
            year: 2020,
        });

        assert_eq!(expected_error, provider.try_into());
    }
}
