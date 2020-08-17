//! Provider for integration-specific data stores

use super::model::{Record, Uid};
use super::{Error, Integration};
use crate::database::{self, Database};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

/// Providable interface for an integration's data store
#[cfg_attr(test, automock)]
pub trait Providable {
    /// Inserts a record for an integration into the database
    ///
    /// # Errors
    ///
    /// When the insertion fails
    fn set<'a>(
        &self,
        integration: &(dyn Integration + 'a),
        uid: Uid,
        data: serde_json::Value,
    ) -> Result<Record, Error>;

    /// Removes an existing record for an integration from the database
    ///
    /// # Errors
    ///
    /// When the removal fails
    fn remove<'a>(&self, integration: &(dyn Integration + 'a), uid: Uid) -> Result<(), Error>;

    /// Retrieves an integration record for a user
    ///
    /// # Errors
    ///
    /// When record retrieval fails
    fn get<'a>(
        &self,
        integration: &(dyn Integration + 'a),
        uid: Uid,
    ) -> Result<serde_json::Value, Error>;
}

/// Provides access to integration data in persistent storage
pub struct Provider {
    database: Arc<dyn Database>,
}

impl Provider {
    /// Creates a new integration data store provider
    #[must_use]
    pub fn new(database: Arc<dyn Database>) -> Self {
        Self { database }
    }
}

impl Providable for Provider {
    fn set<'a>(
        &self,
        integration: &(dyn Integration + 'a),
        uid: Uid,
        data: serde_json::Value,
    ) -> Result<Record, Error> {
        let record = Record {
            uid: uid.uid(),
            uid_type: uid.r#type(),
            name: integration.name(),
            data: serde_json::to_string(&data)
                .map_err(|error| Error::JSONSerialization(error.to_string()))?,
        };

        diesel::insert_into(database::schema::integrations::table)
            .values(&record)
            .execute(self.database.connection())?;

        Ok(record)
    }

    fn remove<'a>(&self, integration: &(dyn Integration + 'a), uid: Uid) -> Result<(), Error> {
        let key = (uid.uid(), uid.r#type(), integration.name());

        diesel::delete(database::schema::integrations::table.find(key))
            .execute(self.database.connection())?;

        Ok(())
    }

    fn get<'a>(
        &self,
        integration: &(dyn Integration + 'a),
        uid: Uid,
    ) -> Result<serde_json::Value, Error> {
        let key = (uid.uid(), uid.r#type(), integration.name().to_string());

        let database_result: Result<String, DieselError> = database::schema::integrations::table
            .find(key)
            .select(database::schema::integrations::data)
            .first(self.database.connection());

        match database_result {
            Err(DieselError::NotFound) => Ok(serde_json::Value::Null),
            Ok(record_data) => Ok(serde_json::from_str(&record_data)?),
            Err(error) => Err(error.into()),
        }
    }
}
