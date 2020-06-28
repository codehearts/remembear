use super::model::{NewUser, UpdatedUser, User};
use super::Error;
use crate::database::{self, Database};
use diesel::prelude::*;
use std::sync::Arc;

/// Provides access to user data in persistent storage
pub struct Provider {
    database: Arc<dyn Database>,
}

impl Provider {
    /// Creates a new user data provider
    pub fn new(database: Arc<dyn Database>) -> Self {
        Self { database }
    }

    /// Creates a new user in the database
    ///
    /// # Errors
    ///
    /// When the insertion fails
    pub fn add(&self, user: NewUser) -> Result<User, Error> {
        diesel::insert_into(database::schema::users::table)
            .values(user)
            .execute(self.database.connection())?;

        Ok(database::schema::users::table
            .order(database::schema::users::uid.desc())
            .first(self.database.connection())?)
    }

    /// Updates an existing user in the database
    ///
    /// # Errors
    ///
    /// When the update fails
    pub fn update(&self, user: UpdatedUser) -> Result<User, Error> {
        let uid = user.uid;

        diesel::update(database::schema::users::table.find(user.uid))
            .set(user)
            .execute(self.database.connection())?;

        self.get_by_uid(uid)
    }

    /// Removes an existing user from the database
    ///
    /// # Errors
    ///
    /// When the removal fails
    pub fn remove(&self, uid: i32) -> Result<(), Error> {
        diesel::delete(database::schema::users::table.find(uid))
            .execute(self.database.connection())?;

        Ok(())
    }

    /// Retrieves all users from the database
    ///
    /// # Errors
    ///
    /// When user retrieval fails
    pub fn get_all(&self) -> Result<Vec<User>, Error> {
        Ok(database::schema::users::table.load(self.database.connection())?)
    }

    /// Retrieves user from the database by their uid
    ///
    /// # Errors
    ///
    /// When user retrieval fails
    pub fn get_by_uid(&self, uid: i32) -> Result<User, Error> {
        Ok(database::schema::users::table
            .find(uid)
            .first(self.database.connection())?)
    }
}
