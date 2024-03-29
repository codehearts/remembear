//! Integration tests for user data management

mod common;
mod common_database;

use common::Result;
use remembear::user::model::{NewUser, UpdatedUser, User};
use remembear::user::{provider::Providable, Provider};

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

#[test]
fn it_gets_inserted_users_by_uid() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    provider.add(NewUser {
        name: String::from("Sarah"),
    })?;

    provider.add(NewUser {
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

    assert_eq!(expected_user_1, provider.get_by_uid(1)?);
    assert_eq!(expected_user_2, provider.get_by_uid(2)?);

    Ok(())
}

#[test]
fn it_updates_existing_users() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    // Add users

    provider.add(NewUser {
        name: String::from("Laura"),
    })?;

    provider.add(NewUser {
        name: String::from("Sarah"),
    })?;

    provider.add(NewUser {
        name: String::from("Leland"),
    })?;

    // Update users

    provider.update(UpdatedUser {
        uid: 2,
        name: String::from("Judy"),
    })?;

    provider.update(UpdatedUser {
        uid: 3,
        name: String::from("Bob"),
    })?;

    let expected_users = vec![
        User {
            uid: 1,
            name: String::from("Laura"),
        },
        User {
            uid: 2,
            name: String::from("Judy"),
        },
        User {
            uid: 3,
            name: String::from("Bob"),
        },
    ];

    assert_eq!(expected_users, provider.get_all()?);

    Ok(())
}

#[test]
fn it_returns_updated_user() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    provider.add(NewUser {
        name: String::from("Sarah"),
    })?;

    let updated_user = provider.update(UpdatedUser {
        uid: 1,
        name: String::from("Judy"),
    })?;

    let expected_user = User {
        uid: 1,
        name: String::from("Judy"),
    };

    assert_eq!(expected_user, updated_user);

    Ok(())
}

#[test]
fn it_removes_existing_users() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    provider.add(NewUser {
        name: String::from("Laura"),
    })?;

    provider.add(NewUser {
        name: String::from("Sarah"),
    })?;

    provider.add(NewUser {
        name: String::from("Leland"),
    })?;

    provider.remove(1)?;
    provider.remove(3)?;

    let expected_users = vec![User {
        uid: 2,
        name: String::from("Sarah"),
    }];

    assert_eq!(expected_users, provider.get_all()?);

    Ok(())
}
