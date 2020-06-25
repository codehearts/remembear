use super::model::NewUser;
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
    pub fn add(&self, user: NewUser) -> Result<(), Error> {
        diesel::insert_into(database::schema::users::table)
            .values(user)
            .execute(self.database.connection())?;

        Ok(())
    }
}
