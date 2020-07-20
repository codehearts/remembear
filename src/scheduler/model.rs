//! Data models for a real-time reminder scheduler

use super::Error;
use crate::Reminder;
use chrono::Utc;
use std::collections::BTreeMap;
use tokio::stream::StreamExt;
use tokio::time::{delay_queue, DelayQueue, Duration, Instant};

/// A reminder with an associated scheduler key
struct ScheduledReminder {
    /// Scheduled reminder
    reminder: Reminder,
    /// Scheduler key for this reminder
    key: delay_queue::Key,
}

/// A real-time scheduler for scheduled reminers
pub struct Scheduler {
    /// A mapping of reminder uid to the scheduled reminder
    reminders: BTreeMap<i32, ScheduledReminder>,
    /// Queue of durations until the next scheduled event
    queue: DelayQueue<i32>,
}

impl Scheduler {
    /// Creates a new real-time scheduler for the given reminders
    #[must_use]
    pub fn new(reminders: Vec<Reminder>) -> Self {
        let mut queue = DelayQueue::with_capacity(reminders.len());

        let scheduled_reminders = reminders
            .into_iter()
            .filter_map(|reminder| {
                if let Some(instant) = get_next_instant(&reminder) {
                    let key = queue.insert_at(reminder.uid, instant);
                    Some((reminder.uid, ScheduledReminder { reminder, key }))
                } else {
                    None
                }
            })
            .collect();

        Self {
            queue,
            reminders: scheduled_reminders,
        }
    }

    /// Processes the next scheduled reminder.
    /// Applications will likely want to call `run` instead
    ///
    /// Returns the uid of the scheduled reminder that was processed
    ///
    /// # Errors
    ///
    /// When a reminder is scheduled but the scheduler queue is empty
    pub async fn next(&mut self) -> Result<Option<i32>, Error> {
        if let Some(scheduled_entity) = self.queue.next().await {
            let uid = *scheduled_entity?.get_ref();
            let entity = self.reminders.get_mut(&uid).expect("get");

            // Insert this reminder's next scheduled time into the queue
            if let Some(instant) = get_next_instant(&entity.reminder) {
                entity.key = self.queue.insert_at(uid, instant);
            }

            Ok(Some(uid))
        } else {
            Ok(None)
        }
    }

    /// Runs the scheduler for as long as there are scheduled reminders
    ///
    /// # Errors
    ///
    /// When a reminder is scheduled but the scheduler queue is empty
    pub async fn run(&mut self) -> Result<(), Error> {
        while self.next().await?.is_some() {}
        Ok(())
    }
}

/// Determines the next instant for a reminder to be scheduled, if possible
fn get_next_instant(reminder: &Reminder) -> Option<Instant> {
    reminder
        .schedule
        .get_next_duration(Utc::now())
        .map(|duration| {
            // Convert from chrono's `Duration` to tokio's
            Instant::now() + duration.to_std().unwrap_or_else(|_| Duration::from_secs(0))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Schedule;
    use chrono::{offset::TimeZone, Datelike, Utc, Weekday};

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// Creates a schedule for the given durations from now
    fn schedule_from_now(durations: Vec<chrono::Duration>) -> Schedule {
        Schedule::new(
            vec![(
                Utc::now().weekday(),
                durations
                    .into_iter()
                    .map(|duration| (Utc::now() + duration).time())
                    .collect(),
            )]
            .into_iter()
            .collect(),
            Utc.isoywd(2020, 1, Weekday::Mon).and_hms(0, 0, 0),
            vec![1],
        )
    }

    #[tokio::test]
    async fn it_schedules_multiple_reminders() -> Result<()> {
        let schedule_1 = schedule_from_now(vec![chrono::Duration::milliseconds(5)]);
        eprintln!("schedule 1: {:?}", schedule_1);

        let schedule_2 = schedule_from_now(vec![chrono::Duration::milliseconds(10)]);

        let reminders = vec![
            Reminder {
                uid: 1,
                name: String::from("Reminder 1"),
                schedule: schedule_1,
            },
            Reminder {
                uid: 2,
                name: String::from("Reminder 2"),
                schedule: schedule_2,
            },
        ];

        let mut scheduler = Scheduler::new(reminders);

        // The first reminder should be scheduled, then the second
        assert_eq!(Some(1), scheduler.next().await?);
        assert_eq!(Some(2), scheduler.next().await?);

        Ok(())
    }

    #[tokio::test]
    async fn it_reschedules_reminders_when_they_leave_the_queue() -> Result<()> {
        let schedule = schedule_from_now(vec![
            chrono::Duration::milliseconds(5),
            chrono::Duration::milliseconds(10),
        ]);

        let reminders = vec![Reminder {
            uid: 1,
            name: String::from("Reminder"),
            schedule,
        }];

        let mut scheduler = Scheduler::new(reminders);

        // The reminder should be rescheduled to occur a second time
        assert_eq!(Some(1), scheduler.next().await?);
        assert_eq!(Some(1), scheduler.next().await?);

        Ok(())
    }
}
