use super::model::NewUser;
use crate::database::{self, Database};

/// Provides access to user data in persistent storage
pub struct Provider<TDatabase: Database> {
    database: TDatabase,
}

impl<TDatabase: Database> Provider<TDatabase> {
    /// Creates a new user data provider
    pub fn new(database: TDatabase) -> Self {
        Self { database }
    }

    /// Creates a new user in the database
    ///
    /// # Errors
    ///
    /// When the insertion fails
    pub fn add(&self, user: NewUser) -> Result<(), database::Error> {
        self.database
            .insert_into(database::schema::users::table, user)
    }
}

#[cfg(test)]
mod tests {
    use super::{NewUser, Provider};
    use crate::database::{self, MockDatabase};
    use mockall::predicate::{always, eq};

    #[test]
    fn it_inserts_new_users_into_the_database() {
        let new_user = NewUser {
            name: String::from("Laura"),
        };

        let mut mock_database = MockDatabase::new();
        mock_database
            .expect_insert_into::<database::schema::users::table, NewUser>()
            .with(always(), eq(new_user.clone()))
            .times(1)
            .returning(|_table, _value| Ok(()));

        let user_provider = Provider::new(mock_database);
        assert!(user_provider.add(new_user).is_ok());
    }
}
