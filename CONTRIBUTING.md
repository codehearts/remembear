# :bear: Welcome to Our Neck of the Woods!

Remembear is a Rust library which can be built as a binary application. This guide will get you up to speed on how to contribute

- [:star2: Getting Started](#star2-getting-started)
  - [Helpful Resources](#helpful-resources)
- [:herb: Project Structure](#herb-project-structure)
- [:computer: Development](#computer-development)
  - [Unit Tests](#unit-tests)
  - [Integration Tests](#integration-tests)
- [:incoming_envelope: Submitting Changes](#incoming_envelope-submitting-changes)
- [:phone: Getting in Touch](#phone-getting-in-touch)

## :star2: Getting Started

1. [Install Rust](https://www.rust-lang.org/tools/install), then install `clippy` and `rustfmt` for linting

        rustup component add clippy
        rustup component add rustfmt
1. Install Sqlite3 for the database

        brew install sqlite # macOS
        apt-get install sqlite # Ubuntu/Mint/others
1. Clone remembear

        git clone git@github.com:codehearts/remembear.git
        cd remembear
1. Make sure you can lint and test the project. This will also install Git hooks

        cargo clippy --all-features
        cargo fmt --all -- --check
        cargo test
1. Now all that's left is to run remembear locally!

        cargo run

### Helpful Resources

If you're new to Rust, you may find these resources helpful:

- [Rust Website](https://www.rust-lang.org/learn)  
  See the [Rust Book](https://doc.rust-lang.org/book/) (big read), [Rust by Example](https://doc.rust-lang.org/stable/rust-by-example/) (much more skimmable), and [Rustlings](https://github.com/rust-lang/rustlings/) (quick hands-on learning)
- [Rust Playground](https://play.rust-lang.org)  
  Lets you run Rust in your browser like a scratchpad
- [Rust Community](https://www.rust-lang.org/community)  
  You can always reach out to me ([@codehearts](https://github.com/codehearts)), but the larger Rust community is also available on forums and via chat

## :herb: Project Structure

This project uses a data provider model, with most modules being organized into these three files:

- `error.rs` - Error types specific to the module
- `model.rs` - Models, usually structs, for representing the module's data
- `provider.rs` - Utilizes dependencies like the database to provide model data

The overall project structure is as follows:

- `src/` - Source code
  - `lib.rs` - Library entrypoint
  - `main.rs` - Binary entrypoint 
  - `command/` - Command-line interface module
  - `database/` - Database integration
  - `integration/` - Integrations with external services
  - `reminder/` - Reminder datatypes
  - `schedule/` - Stateless schedule datatype
    - `provider/` - Provides schedule data from the database
      - `model/` - Models for serialized schedule data
  - `scheduler/` - Real-time reminder scheduler
  - `user/` - User datatypes
- `tests/` - Integration tests
  - `assets/` - Integration test assets
  - `common/` - Common integration test functionality
  - `common_database/` - Common functionality for integration tests that need a database
- `migrations/` - Database schemas for use with [Diesel](http://diesel.rs)
- `diesel.toml` - Configurations for [Diesel](http://diesel.rs)
- `remembear.yml` - Default configuration file for remembear

## :computer: Development

Remembear is written with the latest stable version of Rust and makes extensive use of these crates:

- [Diesel](http://diesel.rs) for persistent storage with sqlite3

### Unit Tests

Code changes should be unit tested whenever possible. Place your tests in a `tests` module at the bottom of the file and annotate your test functions with `#[test]`. Tests will have access to private functions and should have a descriptive name beginning with `it_` (my preference, nbd!)

Note that order doesn't matter in assertions, Rust uses "left/right" instead of "expected/actual"

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_adds_2_and_2_to_4() {
        assert_eq!(4, 2 + 2);
    }
}
```

Run unit tests with Cargo:

```rust
cargo test # test everything
cargo test it_adds # tests all functions matching "it_adds"
```

### Integration Tests

Code changes that impact connectivity between modules or which can't be unit tested should have integration tests. These tests go in `tests/` and should have a descriptive rustdoc at the top. If a test is resource intensive or slow, be sure to annotate it with `#[ignored]`

```rust
//! Integration tests for adding two numbers ¯\_(ツ)_/¯

#[test]
#[ignored]
fn it_adds_2_and_2_to_4() {
    assert_eq!(4, 2 + 2);
}
```

Integration tests are run the same as unit tests, but tests annotated with `#[ignored]` must be run like so:

```rust
cargo test -- --ignored # test everything
cargo test it_adds -- --ignored # tests ignored functions matching "it_adds"
```

## :incoming_envelope: Submitting Changes

Once your code is polished and tests are passing, it's time to submit a pull request! When the CI build for your branch passes and a project owner reviews your code (which should happen within a few days), your change will be rebased into the master branch and your contribution complete! Nice work! :sparkling_heart:

## :phone: Getting in Touch

For features or bugs, you can [create a new issue](https://github.com/codehearts/remembear/issues/new) in the tracker

For questions or concerns, feel free to reach out to [@codehearts](https://twitter.com/codehearts) on Twitter or by email!
