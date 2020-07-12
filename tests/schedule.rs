//! Integration tests for schedule management

mod common;
mod common_database;

#[macro_use]
extern crate diesel;

use chrono::{offset::TimeZone, NaiveTime, Utc, Weekday};
use diesel::{connection::SimpleConnection, ExpressionMethods, QueryDsl, RunQueryDsl};
use std::convert::TryInto;
use thiserror::Error;

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
        vec![(
            Weekday::Mon,
            vec![
                NaiveTime::from_hms(1, 23, 45),
                NaiveTime::from_hms(12, 34, 56),
            ],
        )]
        .into_iter()
        .collect(),
        Utc.isoywd(2020, 3, Weekday::Mon).and_hms(0, 0, 0),
        vec![1, 2, 3],
    );

    let database_schedule =
        insert(r#"{"mon":["01:23:45","12:34:56"]}"#, 202003, "[1, 2, 3]")?.try_into()?;

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
