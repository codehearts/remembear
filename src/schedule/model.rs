//! Data models for a stateless weekly schedule

use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use time::{Duration, OffsetDateTime, Time, Weekday};

/// Mapping of weekdays to a list of times
pub type WeeklyTimes = HashMap<Weekday, Vec<Time>>;

/// Sorted mapping of weekdays to a list of times
type SortedWeeklyTimes = BTreeMap<u8, Vec<Time>>;

/// Stateless weekly schedule with support for rotating assignees
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct Schedule {
    /// Scheduled times of day throughout the week
    pub(crate) weekly_times: WeeklyTimes,
    /// Sorted array of scheduled weekdays, for internal use
    sorted_weekdays: SortedWeeklyTimes,
    /// Beginning of the week in which the schedule started
    pub(crate) start_date: OffsetDateTime,
    /// Assignee ids in order of assignment
    pub(crate) assignees: Vec<i32>,
}

impl Schedule {
    /// Creates a new stateless schedule starting from the given week
    ///
    /// # Errors
    ///
    /// When the given start week is invalid or ambiguous
    #[must_use]
    pub fn new(weekly_times: WeeklyTimes, start_date: OffsetDateTime, assignees: Vec<i32>) -> Self {
        let sorted_weekdays = weekly_times
            .iter()
            .map(|(weekday, times)| (weekday.number_from_monday(), times.clone()))
            .collect();

        Schedule {
            weekly_times,
            sorted_weekdays,
            start_date,
            assignees,
        }
    }

    /// Determines the scheduled assignee for the given datetime
    #[must_use]
    pub fn get_assignee(&self, current_time: OffsetDateTime) -> i32 {
        // Number of fully elapsed weeks since the start week
        // This value is capped if it exceeds `usize::MAX`
        let elapsed_weeks =
            usize::try_from((current_time.date() - self.start_date.date()).whole_weeks())
                .unwrap_or(usize::MAX);

        // Number of scheduled times in a full week
        let times_in_full_week: usize = self.weekly_times.values().map(Vec::len).sum();

        // Number of fully elapsed time periods in this week
        let this_week_day = current_time.weekday();
        let times_in_this_week: usize = self
            .sorted_weekdays
            .iter()
            .map(|(week_day, times_in_day)| {
                match week_day.cmp(&this_week_day.number_from_monday()) {
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
        let index = (elapsed_weeks
            .saturating_mul(times_in_full_week)
            .saturating_add(times_in_this_week))
        .saturating_sub(1);

        self.assignees[index % self.assignees.len()]
    }

    /// Calculates the duration until the next time on the schedule
    #[must_use]
    pub fn get_next_duration(&self, current_time: OffsetDateTime) -> Option<Duration> {
        let this_week_day = current_time.weekday().number_from_monday();

        let next_scheduled_day = self
            .sorted_weekdays
            .iter()
            .find(|(week_day, times_in_day)| {
                match week_day.cmp(&&this_week_day) {
                    Ordering::Less => false,
                    Ordering::Equal => {
                        // Check if the next scheduled time is on the same weekday
                        times_in_day.iter().any(|time| *time >= current_time.time())
                    }
                    Ordering::Greater => true,
                }
            })
            .or_else(|| self.sorted_weekdays.iter().next());

        match next_scheduled_day {
            Some((week_day, times_in_day)) => {
                // Determine the number of days that will elapse
                match (i64::from(*week_day) - i64::from(this_week_day)).checked_rem_euclid(7) {
                    // Same day means either the time between now and the next time,
                    // or the first time of the day if it wrapped into the next week
                    Some(days) if days == 0 => times_in_day
                        .iter()
                        .find_map(|time| {
                            Some(*time - current_time.time()).filter(|time| time >= &Duration::ZERO)
                        })
                        .or_else(|| {
                            times_in_day.get(0).and_then(|time| {
                                Duration::days(7).checked_sub(current_time.time() - *time)
                            })
                        }),
                    // Different days means the time between now and the first time of that day
                    Some(days) => times_in_day.get(0).and_then(|time| {
                        Duration::days(days).checked_sub(current_time.time() - *time)
                    }),
                    None => None,
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::{datetime, time};
    use time::Date;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    fn week(year: i32, week: u8) -> Result<OffsetDateTime> {
        Ok(Date::from_iso_week_date(year, week, Weekday::Monday)?
            .midnight()
            .assume_utc())
    }

    #[test]
    fn it_caps_elapsed_weeks() -> Result<()> {
        let schedule = Schedule::new(
            vec![(Weekday::Monday, vec![time!(00:00:00)])]
                .into_iter()
                .collect(),
            week(2020, 3)?,
            vec![1, 2],
        );

        // Times in the past will overflow `usize`, so they will be capped
        assert_eq!(1, schedule.get_assignee(week(2020, 1)?));
        assert_eq!(1, schedule.get_assignee(week(2020, 2)?));
        // 0 elapsed weeks starts with the first assignee
        assert_eq!(1, schedule.get_assignee(week(2020, 3)?));
        // 1 elapsed week continues as normal
        assert_eq!(2, schedule.get_assignee(week(2020, 4)?));

        Ok(())
    }

    #[test]
    fn it_assigns_days_correctly_without_rollover() -> Result<()> {
        let schedule = Schedule::new(
            vec![
                (Weekday::Monday, vec![time!(12:30)]),
                (Weekday::Wednesday, vec![time!(12:30)]),
                (Weekday::Friday, vec![time!(12:30)]),
            ]
            .into_iter()
            .collect(),
            week(2020, 3)?,
            vec![1, 2, 3],
        );

        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-13 12:30:00 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-15 12:29:59 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-15 12:30:00 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-17 12:29:59 UTC)));
        assert_eq!(3, schedule.get_assignee(datetime!(2020-01-17 12:30:00 UTC)));

        Ok(())
    }

    #[test]
    fn it_assigns_days_correctly_with_rollover() -> Result<()> {
        let schedule = Schedule::new(
            vec![
                (Weekday::Monday, vec![time!(12:30)]),
                (Weekday::Wednesday, vec![time!(12:30)]),
                (Weekday::Friday, vec![time!(12:30)]),
            ]
            .into_iter()
            .collect(),
            week(2020, 3)?,
            vec![1, 2],
        );

        // First week
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-13 12:30:00 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-15 12:29:59 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-15 12:30:00 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-17 12:29:59 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-17 12:30:00 UTC)));

        // Second week
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-20 12:29:59 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-20 12:30:00 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-22 12:29:59 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-22 12:30:00 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-24 12:29:59 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-24 12:30:00 UTC)));

        // Start of third week
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-27 12:29:59 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-27 12:30:00 UTC)));

        Ok(())
    }

    #[test]
    fn it_assigns_times_correctly_without_rollover() -> Result<()> {
        let schedule = Schedule::new(
            vec![(
                Weekday::Monday,
                vec![time!(10:30), time!(11:30), time!(12:30)],
            )]
            .into_iter()
            .collect(),
            week(2020, 3)?,
            vec![1, 2, 3],
        );

        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-13 10:30:00 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-13 11:29:59 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-13 11:30:00 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-13 12:29:59 UTC)));
        assert_eq!(3, schedule.get_assignee(datetime!(2020-01-13 12:30:00 UTC)));

        Ok(())
    }

    #[test]
    fn it_assigns_times_correctly_with_rollover() -> Result<()> {
        let schedule = Schedule::new(
            vec![
                (Weekday::Monday, vec![time!(10:30), time!(11:30)]),
                (Weekday::Friday, vec![time!(12:30)]),
            ]
            .into_iter()
            .collect(),
            week(2020, 3)?,
            vec![1, 2],
        );

        // First week
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-13 10:30:00 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-13 11:29:59 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-13 11:30:00 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-17 12:29:59 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-17 12:30:00 UTC)));

        // Second week
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-20 10:29:59 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-20 10:30:00 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-20 11:29:59 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-20 11:30:00 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-24 12:29:59 UTC)));
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-24 12:30:00 UTC)));

        // Start of third week
        assert_eq!(2, schedule.get_assignee(datetime!(2020-01-27 10:29:59 UTC)));
        assert_eq!(1, schedule.get_assignee(datetime!(2020-01-27 10:30:00 UTC)));

        Ok(())
    }

    #[test]
    fn it_returns_none_without_next_duration() -> Result<()> {
        let schedule = Schedule::new(vec![].into_iter().collect(), week(2020, 1)?, vec![]);

        assert_eq!(None, schedule.get_next_duration(OffsetDateTime::now_utc()));
        Ok(())
    }

    #[test]
    fn it_returns_0_when_next_duration_is_now() -> Result<()> {
        let schedule = Schedule::new(
            vec![(Weekday::Monday, vec![time!(12:30)])]
                .into_iter()
                .collect(),
            week(2020, 1)?,
            vec![1],
        );

        assert_eq!(
            Some(Duration::ZERO),
            schedule.get_next_duration(datetime!(2020-01-06 12:30:00 UTC))
        );

        Ok(())
    }

    #[test]
    fn it_returns_remaining_time_when_next_duration_is_same_day() -> Result<()> {
        let schedule = Schedule::new(
            vec![(Weekday::Monday, vec![time!(12:30)])]
                .into_iter()
                .collect(),
            week(2020, 1)?,
            vec![1],
        );

        assert_eq!(
            Some(Duration::minutes(12 * 60 + 30)),
            schedule.get_next_duration(datetime!(2020-01-06 00:00:00 UTC))
        );

        Ok(())
    }

    #[test]
    fn it_returns_1_day_when_next_duration_is_next_day_at_same_time() -> Result<()> {
        let schedule = Schedule::new(
            vec![(Weekday::Monday, vec![time!(12:30)])]
                .into_iter()
                .collect(),
            week(2020, 1)?,
            vec![1],
        );

        assert_eq!(
            Some(Duration::days(1)),
            schedule.get_next_duration(datetime!(2020-01-05 12:30:00 UTC))
        );

        Ok(())
    }

    #[test]
    fn it_returns_time_gap_when_next_duration_is_less_than_24h() -> Result<()> {
        let schedule = Schedule::new(
            vec![(Weekday::Monday, vec![time!(12:30)])]
                .into_iter()
                .collect(),
            week(2020, 1)?,
            vec![1],
        );

        assert_eq!(
            Some(Duration::seconds(60 * 60 * 24 - 1)),
            schedule.get_next_duration(datetime!(2020-01-05 12:30:01 UTC))
        );

        Ok(())
    }

    #[test]
    fn it_returns_time_gap_when_next_duration_wraps_to_next_week() -> Result<()> {
        let schedule = Schedule::new(
            vec![(Weekday::Monday, vec![time!(12:30)])]
                .into_iter()
                .collect(),
            week(2020, 1)?,
            vec![1],
        );

        assert_eq!(
            Some(Duration::seconds(60 * 60 * 24 * 7 - 1)),
            schedule.get_next_duration(datetime!(2020-01-06 12:30:01 UTC))
        );

        Ok(())
    }

    #[test]
    fn it_returns_time_gap_when_next_duration_is_a_future_day_of_the_week() -> Result<()> {
        let schedule = Schedule::new(
            vec![
                (Weekday::Monday, vec![time!(10:30)]),
                (Weekday::Friday, vec![time!(20:30)]),
            ]
            .into_iter()
            .collect(),
            week(2020, 1)?,
            vec![1],
        );

        assert_eq!(
            // Next time is 4 days and 10 hours minus 1 second
            Some(Duration::seconds(60 * 60 * 24 * 4 + 60 * 60 * 10 - 1)),
            schedule.get_next_duration(datetime!(2020-01-06 10:30:01 UTC))
        );

        Ok(())
    }

    #[test]
    fn it_returns_time_gap_when_next_duration_is_a_past_day_of_the_week() -> Result<()> {
        let schedule = Schedule::new(
            vec![
                (Weekday::Monday, vec![time!(10:30)]),
                (Weekday::Friday, vec![time!(20:30)]),
            ]
            .into_iter()
            .collect(),
            week(2020, 1)?,
            vec![1],
        );

        assert_eq!(
            // Next time is 2 days and 14 hours minus 1 second
            Some(Duration::seconds(60 * 60 * 24 * 2 + 60 * 60 * 14 - 1)),
            schedule.get_next_duration(datetime!(2020-01-10 20:30:01 UTC))
        );

        Ok(())
    }
}
