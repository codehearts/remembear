//! Database controller for persistent storage via sqlite

use super::{Database, Error};
use crate::diesel::RunQueryDsl;
use diesel::connection::Connection;
use diesel::insertable::CanInsertInSingleQuery;
use diesel::query_builder::QueryFragment;
use diesel::sqlite::{Sqlite as SqliteBackend, SqliteConnection};

/// Manages sqlite database connections
pub struct Sqlite {
    connection: SqliteConnection,
}

impl Database for Sqlite {
    fn connect(database_url: &str) -> Result<Self, Error> {
        let connection =
            SqliteConnection::establish(database_url).map_err(|source| Error::Connection {
                database_url: database_url.to_string(),
                source,
            })?;

        Ok(Self { connection })
    }

    fn insert_into<TTable: 'static + diesel::Table, TValue: 'static + diesel::Insertable<TTable>>(
        &self,
        table: TTable,
        value: TValue,
    ) -> Result<(), Error>
    where
        <TTable as diesel::QuerySource>::FromClause: QueryFragment<SqliteBackend>,
        <TValue as diesel::Insertable<TTable>>::Values:
            QueryFragment<SqliteBackend> + CanInsertInSingleQuery<SqliteBackend>,
    {
        diesel::insert_into(table)
            .values(value)
            .execute(&self.connection)?;

        Ok(())
    }
}
