//! Integration tests for the command line interface's user commands

mod common;
mod common_command;
mod common_database;

use common::Result;
use common_command::Executor;
use remembear::User;

#[test]
fn it_outputs_added_user() -> Result<()> {
    let executor = Executor::new()?;

    let output = executor.execute(&["remembear", "user", "add", "Laura"])?;
    let expected_output = serde_json::to_string_pretty(&User {
        uid: 1,
        name: String::from("Laura"),
    })?;

    assert_eq!(expected_output, output);

    Ok(())
}

#[test]
fn it_lists_all_users() -> Result<()> {
    let executor = Executor::new()?;

    executor.execute(&["remembear", "user", "add", "Laura"])?;
    executor.execute(&["remembear", "user", "add", "Leland"])?;
    executor.execute(&["remembear", "user", "add", "Sarah"])?;

    let output = executor.execute(&["remembear", "user", "list"])?;

    let expected_output = serde_json::to_string_pretty(&vec![
        User {
            uid: 1,
            name: String::from("Laura"),
        },
        User {
            uid: 2,
            name: String::from("Leland"),
        },
        User {
            uid: 3,
            name: String::from("Sarah"),
        },
    ])?;

    assert_eq!(expected_output, output);

    Ok(())
}

#[test]
fn it_updates_users() -> Result<()> {
    let executor = Executor::new()?;
    executor.execute(&["remembear", "user", "add", "Leland"])?;

    let output = executor.execute(&["remembear", "user", "update", "1", "--name", "Bob"])?;

    let expected_user = User {
        uid: 1,
        name: String::from("Bob"),
    };

    let expected_output = serde_json::to_string_pretty(&expected_user)?;

    assert_eq!(expected_output, output);

    let list_output = executor.execute(&["remembear", "user", "list"])?;

    let expected_list_output = serde_json::to_string_pretty(&vec![expected_user])?;

    assert_eq!(expected_list_output, list_output);

    Ok(())
}

#[test]
fn it_errors_when_updating_invalid_uid() -> Result<()> {
    let executor = Executor::new()?;
    let output = executor.execute(&["remembear", "user", "update", "1", "--name", "Bob"]);

    match output {
        Ok(_) => panic!("Error was not propagated"),
        Err(error) => assert_eq!("Invalid uid 1", error.to_string()),
    };

    Ok(())
}

#[test]
fn it_removes_users() -> Result<()> {
    let executor = Executor::new()?;
    executor.execute(&["remembear", "user", "add", "Leland"])?;

    let output = executor.execute(&["remembear", "user", "remove", "1"])?;

    let expected_output = serde_json::to_string_pretty(&User {
        uid: 1,
        name: String::from("Leland"),
    })?;

    assert_eq!(expected_output, output);

    let list_output = executor.execute(&["remembear", "user", "list"])?;

    let expected_list_output = serde_json::to_string_pretty::<Vec<User>>(&Vec::new())?;

    assert_eq!(expected_list_output, list_output);

    Ok(())
}

#[test]
fn it_errors_when_removing_invalid_uid() -> Result<()> {
    let executor = Executor::new()?;
    let output = executor.execute(&["remembear", "user", "remove", "1"]);

    match output {
        Ok(_) => panic!("Error was not propagated"),
        Err(error) => assert_eq!("Invalid uid 1", error.to_string()),
    };

    Ok(())
}