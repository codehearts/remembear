//! Integration tests for schedule management

mod common;
mod common_database;

#[macro_use]
extern crate diesel;

use diesel::{connection::SimpleConnection, ExpressionMethods, QueryDsl, RunQueryDsl};
use std::convert::TryInto;
use thiserror::Error;
use time::{macros::time, Date, Weekday};

use common::Result;
use remembear::{schedule, Schedule};

table! {
    use diesel::sql_types::{Text, Integer};

    /// Mock table to test schedule reads from
    schedule_test(start_week) {
        schedule -> Text,
        #[sql_name = "startweek"]
        start_week -> Integer,
        assignees -> Text,
    }
}

/// Potential errors during schedule integration testing
#[derive(Debug, Error)]
enum Error {
    #[error("Database error: {0}")]
    Database(Box<dyn std::error::Error>),
    #[error("Diesel error: {0}")]
    Diesel(#[from] diesel::result::Error),
}

/// Inserts a single record into the database and returns the result of selecting it
fn insert(
    schedule: &str,
    start_week: i32,
    assignees: &str,
) -> std::result::Result<schedule::Provider, Error> {
    let database = common_database::new().map_err(Error::Database)?;

    database.connection().batch_execute(
        r#"
        CREATE TABLE schedule_test (
            schedule Text NOT NULL,
            startweek Integer NOT NULL,
            assignees Text NOT NULL
        );
    "#,
    )?;

    diesel::insert_into(schedule_test::table)
        .values(&(
            schedule_test::schedule.eq(schedule),
            schedule_test::start_week.eq(start_week),
            schedule_test::assignees.eq(assignees),
        ))
        .execute(database.connection())?;

    Ok(schedule_test::table
        .order_by(schedule_test::start_week)
        .first::<schedule::Provider>(database.connection())?)
}

#[test]
fn it_reads_schedules_from_sql() -> Result<()> {
    let expected_schedule = Schedule::new(
        vec![(Weekday::Monday, vec![time!(01:23:45), time!(12:34:56)])]
            .into_iter()
            .collect(),
        Date::from_iso_week_date(2020, 3, Weekday::Monday)?
            .midnight()
            .assume_utc(),
        vec![1, 2, 3],
    );

    let database_schedule = insert(
        r#"{"Monday":["01:23:45.0","12:34:56.0"]}"#,
        202003,
        "[1, 2, 3]",
    )?
    .try_into()?;

    assert_eq!(expected_schedule, database_schedule);

    Ok(())
}

#[test]
fn it_fails_to_read_invalid_schedule_times_from_sql() -> Result<()> {
    let database_result = insert(r#"{"mon":["99:99:99"]}"#, 202003, "[1, 2, 3]");

    assert!(matches!(
        database_result,
        Err(Error::Diesel(diesel::result::Error::DeserializationError(
            ..
        )))
    ));

    Ok(())
}

#[test]
fn it_fails_to_read_invalid_schedule_assignees_from_sql() -> Result<()> {
    let database_result = insert(r#"{"mon":["12:34:56"]}"#, 202003, "1");

    assert!(matches!(
        database_result,
        Err(Error::Diesel(diesel::result::Error::DeserializationError(
            ..
        )))
    ));

    Ok(())
}
