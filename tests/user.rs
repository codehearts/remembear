//! Integration tests for user data management

mod common;
mod common_database;

use common::Result;
use remembear::user::model::{NewUser, User};
use remembear::user::Provider;

#[test]
fn it_gets_nothing_without_users() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    assert!(provider.get_all()?.is_empty());

    Ok(())
}

#[test]
fn it_returns_inserted_users_on_insertion() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    let new_user_1 = provider.add(NewUser {
        name: String::from("Sarah"),
    })?;

    let new_user_2 = provider.add(NewUser {
        name: String::from("Leland"),
    })?;

    let expected_user_1 = User {
        uid: 1,
        name: String::from("Sarah"),
    };
    let expected_user_2 = User {
        uid: 2,
        name: String::from("Leland"),
    };

    assert_eq!(expected_user_1, new_user_1);
    assert_eq!(expected_user_2, new_user_2);

    Ok(())
}

#[test]
fn it_gets_inserted_users() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    provider.add(NewUser {
        name: String::from("Sarah"),
    })?;

    provider.add(NewUser {
        name: String::from("Leland"),
    })?;

    let expected_users = vec![
        User {
            uid: 1,
            name: String::from("Sarah"),
        },
        User {
            uid: 2,
            name: String::from("Leland"),
        },
    ];

    assert_eq!(expected_users, provider.get_all()?);

    Ok(())
}
