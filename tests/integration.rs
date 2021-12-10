//! Integration tests for integrations management

mod common;
mod common_database;

use chrono::{DateTime, Utc};
use common::Result;
use remembear::integration::model::{Record, Uid};
use remembear::integration::Integration;
use remembear::integration::{provider::Providable, Provider};
use remembear::{Providers, Reminder, User};

struct MockIntegration();

impl Integration for MockIntegration {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn execute(&self, _providers: Providers, _arguments: Vec<String>) -> Result<String> {
        Ok(String::from("mock_result"))
    }

    fn notify(
        &mut self,
        _providers: &Providers,
        _reminder: &Reminder,
        _assignees: &[User],
        _timestamp: &DateTime<Utc>,
    ) -> Result<()> {
        Ok(())
    }
}

#[test]
fn it_gets_nothing_when_record_does_not_exist() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);
    let mock_integration = MockIntegration {};

    let data = provider.get(&mock_integration, Uid::User(1))?;

    assert_eq!(serde_json::Value::Null, data);

    Ok(())
}

#[test]
fn it_returns_inserted_records_on_insertion() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);
    let mock_integration = MockIntegration {};

    let new_record_1 = provider.set(
        &mock_integration,
        Uid::User(1),
        serde_json::json!({"key": "value"}),
    )?;

    let new_record_2 = provider.set(
        &mock_integration,
        Uid::User(2),
        serde_json::json!({"value": "key"}),
    )?;

    let expected_record_1 = Record {
        uid: 1,
        uid_type: "user",
        name: "mock",
        data: String::from(r#"{"key":"value"}"#),
    };
    let expected_record_2 = Record {
        uid: 2,
        uid_type: "user",
        name: "mock",
        data: String::from(r#"{"value":"key"}"#),
    };

    assert_eq!(expected_record_1, new_record_1);
    assert_eq!(expected_record_2, new_record_2);

    Ok(())
}

#[test]
fn it_gets_inserted_records() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);
    let mock_integration = MockIntegration {};

    provider.set(
        &mock_integration,
        Uid::User(1),
        serde_json::json!({"key": "value"}),
    )?;

    provider.set(
        &mock_integration,
        Uid::User(2),
        serde_json::json!({"value": "key"}),
    )?;

    let record_data_1 = provider.get(&mock_integration, Uid::User(1))?;
    let record_data_2 = provider.get(&mock_integration, Uid::User(2))?;

    assert_eq!(serde_json::json!({"key": "value"}), record_data_1);
    assert_eq!(serde_json::json!({"value": "key"}), record_data_2);

    Ok(())
}

#[test]
fn it_removes_existing_records() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);
    let mock_integration = MockIntegration {};

    provider.set(&mock_integration, Uid::User(1), serde_json::json!({"A": 1}))?;

    provider.set(&mock_integration, Uid::User(2), serde_json::json!({"B": 2}))?;

    provider.set(&mock_integration, Uid::User(3), serde_json::json!({"C": 3}))?;

    provider.remove(&mock_integration, Uid::User(1))?;
    provider.remove(&mock_integration, Uid::User(3))?;

    let record_data_1 = provider.get(&mock_integration, Uid::User(1))?;
    let record_data_2 = provider.get(&mock_integration, Uid::User(2))?;
    let record_data_3 = provider.get(&mock_integration, Uid::User(3))?;

    assert_eq!(serde_json::Value::Null, record_data_1);
    assert_eq!(serde_json::json!({"B": 2}), record_data_2);
    assert_eq!(serde_json::Value::Null, record_data_3);

    Ok(())
}
