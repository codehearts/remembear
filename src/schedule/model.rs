//! Data models for a stateless weekly schedule

use chrono::{DateTime, Datelike, NaiveTime, Utc, Weekday};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;

/// Mapping of weekdays to a list of times
pub type WeeklyTimes = HashMap<Weekday, Vec<NaiveTime>>;

/// Stateless weekly schedule with support for rotating assignees
#[derive(Debug, PartialEq, Serialize)]
pub struct Schedule {
    /// Scheduled times of day throughout the week
    weekly_times: WeeklyTimes,
    /// Beginning of the week in which the schedule started
    start_date: DateTime<Utc>,
    /// Assignee ids in order of assignment
    assignees: Vec<i32>,
}

impl Schedule {
    /// Creates a new stateless schedule starting from the given week
    ///
    /// # Errors
    ///
    /// When the given start week is invalid or ambiguous
    #[must_use]
    pub fn new(weekly_times: WeeklyTimes, start_date: DateTime<Utc>, assignees: Vec<i32>) -> Self {
        Schedule {
            weekly_times,
            start_date,
            assignees,
        }
    }

    /// Determines the scheduled assignee for the given datetime
    #[must_use]
    pub fn get_assignee(&self, current_time: DateTime<Utc>) -> i32 {
        // Number of fully elapsed weeks since the start week
        // This value is capped if it exceeds `usize::MAX`
        let elapsed_weeks = usize::try_from(
            current_time
                .date()
                .signed_duration_since(self.start_date.date())
                .num_weeks(),
        )
        .unwrap_or(usize::MAX);

        // Number of scheduled times in a full week
        let times_in_full_week: usize = self.weekly_times.values().map(Vec::len).sum();

        // Number of fully elapsed time periods in this week
        let this_week_day = current_time.weekday();
        let times_in_this_week: usize = self
            .weekly_times
            .iter()
            .map(|(week_day, times_in_day)| {
                match week_day
                    .number_from_monday()
                    .cmp(&this_week_day.number_from_monday())
                {
                    Ordering::Less => times_in_day.len(),
                    Ordering::Equal => times_in_day
                        .iter()
                        .take_while(|time| *time <= &current_time.time())
                        .count(),
                    Ordering::Greater => 0,
                }
            })
            .sum();

        // Subtract 1 from the value to obtain an array index
        let index = (elapsed_weeks * times_in_full_week + times_in_this_week).saturating_sub(1);

        self.assignees[index % self.assignees.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    fn time(hour: u32, minute: u32) -> NaiveTime {
        NaiveTime::from_hms(hour, minute, 0)
    }

    fn week(year: i32, week: u32) -> DateTime<Utc> {
        Utc.isoywd(year, week, Weekday::Mon).and_hms(0, 0, 0)
    }

    fn datetime(datetime: &str) -> Result<DateTime<Utc>> {
        Ok(format!("{}Z", datetime).as_str().parse()?)
    }

    #[test]
    fn it_assigns_days_correctly_without_rollover() -> Result<()> {
        let schedule = Schedule::new(
            vec![
                (Weekday::Mon, vec![time(12, 30)]),
                (Weekday::Wed, vec![time(12, 30)]),
                (Weekday::Fri, vec![time(12, 30)]),
            ]
            .into_iter()
            .collect(),
            week(2020, 3),
            vec![1, 2, 3],
        );

        assert_eq!(1, schedule.get_assignee(datetime("2020-01-13 12:30:00")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-15 12:29:59")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-15 12:30:00")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-17 12:29:59")?));
        assert_eq!(3, schedule.get_assignee(datetime("2020-01-17 12:30:00")?));

        Ok(())
    }

    #[test]
    fn it_assigns_days_correctly_with_rollover() -> Result<()> {
        let schedule = Schedule::new(
            vec![
                (Weekday::Mon, vec![time(12, 30)]),
                (Weekday::Wed, vec![time(12, 30)]),
                (Weekday::Fri, vec![time(12, 30)]),
            ]
            .into_iter()
            .collect(),
            week(2020, 3),
            vec![1, 2],
        );

        // First week
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-13 12:30:00")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-15 12:29:59")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-15 12:30:00")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-17 12:29:59")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-17 12:30:00")?));

        // Second week
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-20 12:29:59")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-20 12:30:00")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-22 12:29:59")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-22 12:30:00")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-24 12:29:59")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-24 12:30:00")?));

        // Start of third week
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-27 12:29:59")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-27 12:30:00")?));

        Ok(())
    }

    #[test]
    fn it_assigns_times_correctly_without_rollover() -> Result<()> {
        let schedule = Schedule::new(
            vec![(Weekday::Mon, vec![time(10, 30), time(11, 30), time(12, 30)])]
                .into_iter()
                .collect(),
            week(2020, 3),
            vec![1, 2, 3],
        );

        assert_eq!(1, schedule.get_assignee(datetime("2020-01-13 10:30:00")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-13 11:29:59")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-13 11:30:00")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-13 12:29:59")?));
        assert_eq!(3, schedule.get_assignee(datetime("2020-01-13 12:30:00")?));

        Ok(())
    }

    #[test]
    fn it_assigns_times_correctly_with_rollover() -> Result<()> {
        let schedule = Schedule::new(
            vec![
                (Weekday::Mon, vec![time(10, 30), time(11, 30)]),
                (Weekday::Fri, vec![time(12, 30)]),
            ]
            .into_iter()
            .collect(),
            week(2020, 3),
            vec![1, 2],
        );

        // First week
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-13 10:30:00")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-13 11:29:59")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-13 11:30:00")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-17 12:29:59")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-17 12:30:00")?));

        // Second week
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-20 10:29:59")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-20 10:30:00")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-20 11:29:59")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-20 11:30:00")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-24 12:29:59")?));
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-24 12:30:00")?));

        // Start of third week
        assert_eq!(2, schedule.get_assignee(datetime("2020-01-27 10:29:59")?));
        assert_eq!(1, schedule.get_assignee(datetime("2020-01-27 10:30:00")?));

        Ok(())
    }
}
