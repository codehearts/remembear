//! Provider for user data

use super::model::{NewUser, UpdatedUser, User};
use super::Error;
use crate::database::{self, Database};
use diesel::prelude::*;
use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

/// Provides an interface for user management
#[cfg_attr(test, automock)]
pub trait UserManagement {
    /// Creates a new user in the database
    ///
    /// # Errors
    ///
    /// When the insertion fails
    fn add(&self, user: NewUser) -> Result<User, Error>;

    /// Updates an existing user in the database
    ///
    /// # Errors
    ///
    /// When the update fails
    fn update(&self, user: UpdatedUser) -> Result<User, Error>;

    /// Removes an existing user from the database
    ///
    /// # Errors
    ///
    /// When the removal fails
    fn remove(&self, uid: i32) -> Result<(), Error>;

    /// Retrieves all users from the database
    ///
    /// # Errors
    ///
    /// When user retrieval fails
    fn get_all(&self) -> Result<Vec<User>, Error>;

    /// Retrieves user from the database by their uid
    ///
    /// # Errors
    ///
    /// When user retrieval fails
    fn get_by_uid(&self, uid: i32) -> Result<User, Error>;
}

/// Provides access to user data in persistent storage
pub struct Provider {
    database: Arc<dyn Database>,
}

impl Provider {
    /// Creates a new user data provider
    #[must_use]
    pub fn new(database: Arc<dyn Database>) -> Self {
        Self { database }
    }
}

impl UserManagement for Provider {
    fn add(&self, user: NewUser) -> Result<User, Error> {
        diesel::insert_into(database::schema::users::table)
            .values(user)
            .execute(self.database.connection())?;

        Ok(database::schema::users::table
            .order(database::schema::users::uid.desc())
            .first(self.database.connection())?)
    }

    fn update(&self, user: UpdatedUser) -> Result<User, Error> {
        let uid = user.uid;

        diesel::update(database::schema::users::table.find(user.uid))
            .set(user)
            .execute(self.database.connection())?;

        self.get_by_uid(uid)
    }

    fn remove(&self, uid: i32) -> Result<(), Error> {
        diesel::delete(database::schema::users::table.find(uid))
            .execute(self.database.connection())?;

        Ok(())
    }

    fn get_all(&self) -> Result<Vec<User>, Error> {
        Ok(database::schema::users::table.load(self.database.connection())?)
    }

    fn get_by_uid(&self, uid: i32) -> Result<User, Error> {
        Ok(database::schema::users::table
            .find(uid)
            .first(self.database.connection())?)
    }
}
