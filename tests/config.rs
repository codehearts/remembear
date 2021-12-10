//! Integration tests for loading config files into memory

use remembear::config::{self, error::Error, Config};

#[test]
fn it_deserializes_valid_configs_to_correct_type() {
    let expected_config = config::Config {
        database: config::Database {
            sqlite: config::SqliteDatabase {
                path: String::from("valid.sqlite3"),
            },
        },
        integrations: Some(
            vec![(
                String::from("console"),
                vec![(String::from("enabled"), String::from("true"))]
                    .into_iter()
                    .collect(),
            )]
            .into_iter()
            .collect(),
        ),
    };

    let load_result = Config::load("tests/assets/valid_config");

    if let Ok(actual_config) = load_result {
        assert_eq!(expected_config, actual_config);
    } else if let Err(error) = load_result {
        panic!("Failed to load valid config: {}", error);
    }
}

#[test]
fn it_returns_file_read_error_for_missing_file() {
    let missing_filename = "tests/assets/missing_config";
    let load_result = Config::load(missing_filename);

    match load_result {
        Err(Error::FileRead { filename, .. }) => assert_eq!(missing_filename, filename),
        _ => panic!("FileRead wasn't returned, got {:?}", load_result),
    }
}

#[test]
fn it_returns_invalid_syntax_error_for_invalid_config() {
    let invalid_syntax_filename = "tests/assets/invalid_config";
    let load_result = Config::load(invalid_syntax_filename);

    match load_result {
        Err(Error::InvalidSyntax { filename, .. }) => assert_eq!(invalid_syntax_filename, filename),
        _ => panic!("InvalidSyntax wasn't returned, got {:?}", load_result),
    }
}
