//! Integration tests for the command line interface's reminder commands

mod common;
mod common_command;
mod common_database;

use chrono::{DateTime, Datelike, TimeZone, Utc, Weekday};
use common::Result;
use common_command::Executor;
use remembear::{Reminder, Schedule};

/// Returns the start of the current week
fn get_start_of_this_week() -> DateTime<Utc> {
    let iso_week = Utc::today().iso_week();
    Utc.isoywd(iso_week.year(), iso_week.week(), Weekday::Mon)
        .and_hms(0, 0, 0)
}

#[tokio::test]
async fn it_outputs_added_reminder() -> Result<()> {
    let executor = Executor::new()?;
    let schedule = r#"{"mon":["21:00:00"]}"#;

    let output = executor
        .execute(&[
            "remembear",
            "reminder",
            "add",
            "Meet at Roadhouse",
            schedule,
            "1",
            "2",
        ])
        .await?;

    let expected_output = serde_json::to_string_pretty(&Reminder {
        uid: 1,
        name: String::from("Meet at Roadhouse"),
        schedule: Schedule::new(
            serde_json::from_str(schedule)?,
            get_start_of_this_week(),
            vec![1, 2],
        ),
    })?;

    assert_eq!(expected_output, output);

    Ok(())
}

#[tokio::test]
async fn it_lists_all_reminders() -> Result<()> {
    let executor = Executor::new()?;
    let schedule_1 = r#"{"mon":["21:00:00"]}"#;
    let schedule_2 = r#"{"wed":["14:53:00"]}"#;

    executor
        .execute(&["remembear", "reminder", "add", "Roadhouse", schedule_1, "1"])
        .await?;
    executor
        .execute(&["remembear", "reminder", "add", "2:53", schedule_2, "2"])
        .await?;

    let output = executor.execute(&["remembear", "reminder", "list"]).await?;

    let expected_output = serde_json::to_string_pretty(&vec![
        Reminder {
            uid: 1,
            name: String::from("Roadhouse"),
            schedule: Schedule::new(
                serde_json::from_str(schedule_1)?,
                get_start_of_this_week(),
                vec![1],
            ),
        },
        Reminder {
            uid: 2,
            name: String::from("2:53"),
            schedule: Schedule::new(
                serde_json::from_str(schedule_2)?,
                get_start_of_this_week(),
                vec![2],
            ),
        },
    ])?;

    assert_eq!(expected_output, output);

    Ok(())
}

#[tokio::test]
async fn it_updates_reminders() -> Result<()> {
    let executor = Executor::new()?;
    let old_schedule = r#"{"mon":["21:00:00"]}"#;
    let new_schedule = r#"{"wed":["21:30:00"]}"#;

    executor
        .execute(&["remembear", "reminder", "add", "Roadhouse", old_schedule])
        .await?;

    let output = executor
        .execute(&[
            "remembear",
            "reminder",
            "update",
            "1",
            "--name",
            "Meet at Roadhouse",
            "-s",
            new_schedule,
            "-a3",
            "-a4",
        ])
        .await?;

    let expected_reminder = Reminder {
        uid: 1,
        name: String::from("Meet at Roadhouse"),
        schedule: Schedule::new(
            serde_json::from_str(new_schedule)?,
            get_start_of_this_week(),
            vec![3, 4],
        ),
    };

    let expected_output = serde_json::to_string_pretty(&expected_reminder)?;

    assert_eq!(expected_output, output);

    let list_output = executor.execute(&["remembear", "reminder", "list"]).await?;

    let expected_list_output = serde_json::to_string_pretty(&vec![expected_reminder])?;

    assert_eq!(expected_list_output, list_output);

    Ok(())
}

#[tokio::test]
async fn it_errors_when_updating_invalid_uid() -> Result<()> {
    let executor = Executor::new()?;
    let output = executor
        .execute(&["remembear", "reminder", "update", "1"])
        .await
        .map_err(|error| error.to_string());

    assert_eq!(Some(String::from("Invalid uid 1")), output.err());

    Ok(())
}

#[tokio::test]
async fn it_removes_reminders() -> Result<()> {
    let executor = Executor::new()?;
    let schedule = r#"{"mon":["21:00:00"]}"#;

    executor
        .execute(&["remembear", "reminder", "add", "Roadhouse", schedule, "1"])
        .await?;

    let output = executor
        .execute(&["remembear", "reminder", "remove", "1"])
        .await?;

    let expected_output = serde_json::to_string_pretty(&Reminder {
        uid: 1,
        name: String::from("Roadhouse"),
        schedule: Schedule::new(
            serde_json::from_str(schedule)?,
            get_start_of_this_week(),
            vec![1],
        ),
    })?;

    assert_eq!(expected_output, output);

    let list_output = executor.execute(&["remembear", "reminder", "list"]).await?;

    let expected_list_output = serde_json::to_string_pretty::<Vec<Reminder>>(&Vec::new())?;

    assert_eq!(expected_list_output, list_output);

    Ok(())
}

#[tokio::test]
async fn it_errors_when_removing_invalid_uid() -> Result<()> {
    let executor = Executor::new()?;
    let output = executor
        .execute(&["remembear", "reminder", "remove", "1"])
        .await
        .map_err(|error| error.to_string());

    assert_eq!(Some(String::from("Invalid uid 1")), output.err());

    Ok(())
}
