//! Data models for a real-time reminder scheduler

use super::Error;
use crate::{Integrations, Providers, Reminder};
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
pub struct Scheduler<'a> {
    /// A mapping of reminder uid to the scheduled reminder
    reminders: BTreeMap<i32, ScheduledReminder>,
    /// Providers for scheduling data
    providers: Providers<'a>,
    /// Integrations for the scheduler
    integrations: Integrations,
    /// Queue of durations until the next scheduled event
    queue: DelayQueue<i32>,
}

impl<'a> Scheduler<'a> {
    /// Creates a new real-time scheduler for the given reminders
    #[must_use]
    pub fn new(
        reminders: Vec<Reminder>,
        providers: Providers<'a>,
        integrations: Integrations,
    ) -> Self {
        let mut queue = DelayQueue::with_capacity(reminders.len());

        let scheduled_reminders = reminders
            .into_iter()
            .filter_map(|reminder| {
                get_next_instant(&reminder).map(|instant| {
                    let key = queue.insert_at(reminder.uid, instant);
                    (reminder.uid, ScheduledReminder { reminder, key })
                })
            })
            .collect();

        Self {
            queue,
            providers,
            integrations,
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
            let entity = self
                .reminders
                .get_mut(&uid)
                .ok_or(Error::Unavailable(uid))?;

            // Notify integrations
            if !self.integrations.is_empty() {
                let timestamp = Utc::now();
                let assignees = vec![self
                    .providers
                    .user
                    .get_by_uid(entity.reminder.schedule.get_assignee(timestamp))?];

                for integration in self.integrations.values_mut() {
                    integration
                        .notify(&self.providers, &entity.reminder, &assignees, &timestamp)
                        .unwrap_or_else(|error| eprintln!("Integration failed: {:?}", error));
                }
            }

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
    use crate::integration::{Integrations, MockIntegration};
    use crate::{Schedule, User};
    use chrono::{offset::TimeZone, DateTime, Datelike, Utc, Weekday};
    use mockall::predicate::*;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// Creates a schedule for the given durations from now
    fn schedule_from_timestamp(
        timestamp: DateTime<Utc>,
        durations: Vec<chrono::Duration>,
    ) -> Schedule {
        Schedule::new(
            vec![(
                timestamp.weekday(),
                durations
                    .into_iter()
                    .map(|duration| (timestamp + duration).time())
                    .collect(),
            )]
            .into_iter()
            .collect(),
            Utc.isoywd(2020, 1, Weekday::Mon).and_hms(0, 0, 0),
            vec![1],
        )
    }

    #[tokio::test]
    async fn it_does_nothing_with_empty_schedules() -> Result<()> {
        let schedule = Schedule::new(
            vec![].into_iter().collect(),
            Utc.isoywd(2020, 1, Weekday::Mon).and_hms(0, 0, 0),
            vec![1],
        );

        let reminder = Reminder {
            uid: 1,
            name: String::from("Reminder"),
            schedule,
        };

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };

        let mut scheduler = Scheduler::new(vec![reminder], providers, Integrations::default());

        assert_eq!(None, scheduler.next().await?);

        Ok(())
    }

    #[tokio::test]
    async fn it_schedules_multiple_reminders() -> Result<()> {
        let schedule_1 =
            schedule_from_timestamp(Utc::now(), vec![chrono::Duration::milliseconds(5)]);

        let schedule_2 =
            schedule_from_timestamp(Utc::now(), vec![chrono::Duration::milliseconds(10)]);

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

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };

        let mut scheduler = Scheduler::new(reminders, providers, Integrations::default());

        // The first reminder should be scheduled, then the second
        assert_eq!(Some(1), scheduler.next().await?);
        assert_eq!(Some(2), scheduler.next().await?);

        Ok(())
    }

    #[tokio::test]
    async fn it_reschedules_reminders_when_they_leave_the_queue() -> Result<()> {
        let schedule = schedule_from_timestamp(
            Utc::now(),
            vec![
                chrono::Duration::milliseconds(5),
                chrono::Duration::milliseconds(10),
            ],
        );

        let reminders = vec![Reminder {
            uid: 1,
            name: String::from("Reminder"),
            schedule,
        }];

        let providers = Providers {
            user: &crate::user::provider::MockProvidable::new(),
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };

        let mut scheduler = Scheduler::new(reminders, providers, Integrations::default());

        // The reminder should be rescheduled to occur a second time
        assert_eq!(Some(1), scheduler.next().await?);
        assert_eq!(Some(1), scheduler.next().await?);

        Ok(())
    }

    /// Returns a user with uid 1 for testing
    fn test_user() -> User {
        User {
            uid: 1,
            name: String::from("Laura"),
        }
    }

    /// Returns a reminder named "Reminder" with uid 1 assigned
    fn test_reminder(timestamp: DateTime<Utc>) -> Reminder {
        let schedule = schedule_from_timestamp(timestamp, vec![chrono::Duration::milliseconds(5)]);

        Reminder {
            uid: 1,
            name: String::from("Reminder"),
            schedule,
        }
    }

    #[tokio::test]
    async fn it_notifies_integrations_with_reminders() -> Result<()> {
        let current_timestamp = Utc::now();

        let mut mock_user_provider = crate::user::provider::MockProvidable::new();
        mock_user_provider
            .expect_get_by_uid()
            .with(eq(1))
            .returning(|_| Ok(test_user()))
            .times(1);

        let providers = Providers {
            user: &mock_user_provider,
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };

        let mut mock_integration = MockIntegration::new();

        // Expect a timestamp in the future because we run the scheduler next
        mock_integration
            .expect_notify()
            .with(
                always(),
                eq(test_reminder(current_timestamp)),
                function(|users: &[User]| users[0] == test_user()),
                gt(Utc::now()),
            )
            .returning(|_, _, _, _| Ok(()))
            .times(1);

        let mut integrations = Integrations::default();
        integrations.insert("mock", Box::new(mock_integration));

        let mut scheduler = Scheduler::new(
            vec![test_reminder(current_timestamp)],
            providers,
            integrations,
        );

        // Run the scheduler for one tick
        scheduler.next().await?;

        Ok(())
    }

    #[tokio::test]
    async fn it_continues_when_an_integration_fails() -> Result<()> {
        let current_timestamp = Utc::now();

        let mut mock_user_provider = crate::user::provider::MockProvidable::new();
        mock_user_provider
            .expect_get_by_uid()
            .with(eq(1))
            .returning(|_| Ok(test_user()))
            .times(1);

        let providers = Providers {
            user: &mock_user_provider,
            reminder: &crate::reminder::provider::MockProvidable::new(),
            integration: &crate::integration::provider::MockProvidable::new(),
        };

        let mut mock_integration = MockIntegration::new();

        // Expect a timestamp in the future because we run the scheduler next
        mock_integration
            .expect_notify()
            .with(
                always(),
                eq(test_reminder(current_timestamp)),
                function(|users: &[User]| users[0] == test_user()),
                gt(Utc::now()),
            )
            .returning(|_, _, _, _| Err(Box::new(Error::Unavailable(1))))
            .times(1);

        let mut integrations = Integrations::default();
        integrations.insert("mock", Box::new(mock_integration));

        let mut scheduler = Scheduler::new(
            vec![test_reminder(current_timestamp)],
            providers,
            integrations,
        );

        // Run the scheduler for one tick, which should return Ok
        scheduler.next().await?;

        Ok(())
    }
}
