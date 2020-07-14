//! Integration tests for reminder management

mod common;
mod common_database;

use chrono::{NaiveTime, TimeZone, Utc, Weekday};

use common::Result;
use remembear::reminder::model::{NewReminder, Reminder, UpdatedReminder};
use remembear::reminder::{provider::Providable, Provider};
use remembear::Schedule;

fn get_roadhouse_schedule() -> Schedule {
    Schedule::new(
        vec![(Weekday::Mon, vec![NaiveTime::from_hms(21, 0, 0)])]
            .into_iter()
            .collect(),
        Utc.isoywd(2020, 1, Weekday::Mon).and_hms(0, 0, 0),
        vec![1, 2],
    )
}

fn get_253_schedule() -> Schedule {
    Schedule::new(
        vec![(Weekday::Wed, vec![NaiveTime::from_hms(14, 53, 0)])]
            .into_iter()
            .collect(),
        Utc.isoywd(2020, 2, Weekday::Mon).and_hms(0, 0, 0),
        vec![3, 4],
    )
}

fn get_254_schedule() -> Schedule {
    Schedule::new(
        vec![(Weekday::Wed, vec![NaiveTime::from_hms(14, 54, 0)])]
            .into_iter()
            .collect(),
        Utc.isoywd(2020, 2, Weekday::Mon).and_hms(0, 0, 0),
        vec![3, 4],
    )
}

fn get_lodge_schedule() -> Schedule {
    Schedule::new(
        vec![(Weekday::Wed, vec![NaiveTime::from_hms(14, 53, 0)])]
            .into_iter()
            .collect(),
        Utc.isoywd(1989, 13, Weekday::Mon).and_hms(0, 0, 0),
        vec![1, 2, 3],
    )
}

#[test]
fn it_gets_nothing_without_reminders() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    assert!(provider.get_all()?.is_empty());

    Ok(())
}

#[test]
fn it_returns_inserted_reminders_on_insertion() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    let new_reminder_1 = provider.add(NewReminder {
        name: String::from("Meet at Roadhouse"),
        schedule: get_roadhouse_schedule(),
    })?;

    let new_reminder_2 = provider.add(NewReminder {
        name: String::from("2:53"),
        schedule: get_253_schedule(),
    })?;

    let expected_reminder_1 = Reminder {
        uid: 1,
        name: String::from("Meet at Roadhouse"),
        schedule: get_roadhouse_schedule(),
    };
    let expected_reminder_2 = Reminder {
        uid: 2,
        name: String::from("2:53"),
        schedule: get_253_schedule(),
    };

    assert_eq!(expected_reminder_1, new_reminder_1);
    assert_eq!(expected_reminder_2, new_reminder_2);

    Ok(())
}

#[test]
fn it_gets_inserted_reminders() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    provider.add(NewReminder {
        name: String::from("Meet at Roadhouse"),
        schedule: get_roadhouse_schedule(),
    })?;

    provider.add(NewReminder {
        name: String::from("2:53"),
        schedule: get_253_schedule(),
    })?;

    let expected_reminders = vec![
        Reminder {
            uid: 1,
            name: String::from("Meet at Roadhouse"),
            schedule: get_roadhouse_schedule(),
        },
        Reminder {
            uid: 2,
            name: String::from("2:53"),
            schedule: get_253_schedule(),
        },
    ];

    assert_eq!(expected_reminders, provider.get_all()?);

    Ok(())
}

#[test]
fn it_gets_inserted_reminders_by_uid() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    provider.add(NewReminder {
        name: String::from("Meet at Roadhouse"),
        schedule: get_roadhouse_schedule(),
    })?;

    provider.add(NewReminder {
        name: String::from("2:53"),
        schedule: get_253_schedule(),
    })?;

    let expected_reminder_1 = Reminder {
        uid: 1,
        name: String::from("Meet at Roadhouse"),
        schedule: get_roadhouse_schedule(),
    };
    let expected_reminder_2 = Reminder {
        uid: 2,
        name: String::from("2:53"),
        schedule: get_253_schedule(),
    };

    assert_eq!(expected_reminder_1, provider.get_by_uid(1)?);
    assert_eq!(expected_reminder_2, provider.get_by_uid(2)?);

    Ok(())
}

#[test]
fn it_updates_existing_reminders() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    // Add reminders

    provider.add(NewReminder {
        name: String::from("Meet at Roadhouse"),
        schedule: get_roadhouse_schedule(),
    })?;

    provider.add(NewReminder {
        name: String::from("2:53"),
        schedule: get_253_schedule(),
    })?;

    // Update reminders

    provider.update(UpdatedReminder {
        uid: 1,
        name: String::from("Meet Donna at Roadhouse"),
        schedule: get_roadhouse_schedule(),
    })?;

    provider.update(UpdatedReminder {
        uid: 2,
        name: String::from("2:54"),
        schedule: get_254_schedule(),
    })?;

    let expected_reminders = vec![
        Reminder {
            uid: 1,
            name: String::from("Meet Donna at Roadhouse"),
            schedule: get_roadhouse_schedule(),
        },
        Reminder {
            uid: 2,
            name: String::from("2:54"),
            schedule: get_254_schedule(),
        },
    ];

    assert_eq!(expected_reminders, provider.get_all()?);

    Ok(())
}

#[test]
fn it_returns_updated_reminder() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    provider.add(NewReminder {
        name: String::from("2:53"),
        schedule: get_253_schedule(),
    })?;

    let updated_reminder = provider.update(UpdatedReminder {
        uid: 1,
        name: String::from("2:54"),
        schedule: get_254_schedule(),
    })?;

    let expected_reminder = Reminder {
        uid: 1,
        name: String::from("2:54"),
        schedule: get_254_schedule(),
    };

    assert_eq!(expected_reminder, updated_reminder);

    Ok(())
}

#[test]
fn it_removes_existing_reminders() -> Result<()> {
    let database = common_database::new()?;
    let provider = Provider::new(database);

    provider.add(NewReminder {
        name: String::from("Meet at Roadhouse"),
        schedule: get_roadhouse_schedule(),
    })?;

    provider.add(NewReminder {
        name: String::from("2:53"),
        schedule: get_253_schedule(),
    })?;

    provider.add(NewReminder {
        name: String::from("Black Lodge Opens"),
        schedule: get_lodge_schedule(),
    })?;

    provider.remove(1)?;
    provider.remove(3)?;

    let expected_reminders = vec![Reminder {
        uid: 2,
        name: String::from("2:53"),
        schedule: get_253_schedule(),
    }];

    assert_eq!(expected_reminders, provider.get_all()?);

    Ok(())
}
