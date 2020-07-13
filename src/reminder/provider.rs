//! Provider for reminder data

use super::model::{InsertableNewReminder, NewReminder, Reminder, UpdatedReminder};
use super::Error;
use crate::database::{schema::reminders, Database};
use crate::schedule;
use diesel::prelude::*;
use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

/// Providable interface for reminder management
#[cfg_attr(test, automock)]
pub trait Providable {
    /// Creates a new reminder in the database
    ///
    /// # Errors
    ///
    /// When the insertion fails
    fn add(&self, reminder: NewReminder) -> Result<Reminder, Error>;

    /// Updates an existing reminder in the database
    ///
    /// # Errors
    ///
    /// When the update fails
    fn update(&self, reminder: UpdatedReminder) -> Result<Reminder, Error>;

    /// Removes an existing reminder from the database
    ///
    /// # Errors
    ///
    /// When the removal fails
    fn remove(&self, uid: i32) -> Result<(), Error>;

    /// Retrieves all reminders from the database
    ///
    /// # Errors
    ///
    /// When reminder retrieval fails
    fn get_all(&self) -> Result<Vec<Reminder>, Error>;

    /// Retrieves a reminder from the database by its uid
    ///
    /// # Errors
    ///
    /// When user retrieval fails
    fn get_by_uid(&self, uid: i32) -> Result<Reminder, Error>;
}

/// Provides access to reminder data in persistent storage
pub struct Provider {
    database: Arc<dyn Database>,
}

impl Provider {
    /// Creates a new reminder data provider
    #[must_use]
    pub fn new(database: Arc<dyn Database>) -> Self {
        Self { database }
    }
}

impl Providable for Provider {
    fn add(&self, reminder: NewReminder) -> Result<Reminder, Error> {
        let insertable_reminder: InsertableNewReminder = reminder.into();

        diesel::insert_into(reminders::table)
            .values(insertable_reminder)
            .execute(self.database.connection())?;

        Ok(reminders::table
            .order(reminders::uid.desc())
            .first(self.database.connection())?)
    }

    fn update(&self, reminder: UpdatedReminder) -> Result<Reminder, Error> {
        let uid = reminder.uid;
        let schedule: schedule::Provider = reminder.schedule.into();

        diesel::update(reminders::table.find(reminder.uid))
            .set((
                reminders::columns::name.eq(reminder.name),
                reminders::columns::schedule.eq(schedule.weekly_times),
                reminders::columns::start_week.eq(schedule.start_week),
                reminders::columns::assignees.eq(schedule.assignees),
            ))
            .execute(self.database.connection())?;

        self.get_by_uid(uid)
    }

    fn remove(&self, uid: i32) -> Result<(), Error> {
        diesel::delete(reminders::table.find(uid)).execute(self.database.connection())?;

        Ok(())
    }

    fn get_all(&self) -> Result<Vec<Reminder>, Error> {
        Ok(reminders::table.load(self.database.connection())?)
    }

    fn get_by_uid(&self, uid: i32) -> Result<Reminder, Error> {
        Ok(reminders::table
            .find(uid)
            .first(self.database.connection())?)
    }
}
