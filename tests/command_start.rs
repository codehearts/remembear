//! Integration tests for the command line interface's scheduler commands

mod common;
mod common_command;
mod common_database;

use common::Result;
use common_command::Executor;

#[tokio::test]
async fn it_starts_scheduler() -> Result<()> {
    let executor = Executor::new()?;
    let output = executor.execute(&["remembear", "start"]).await?;

    // There are no reminders in the test database, so the queue will be empty
    assert_eq!(String::from("Scheduler queue is empty"), output);

    Ok(())
}
