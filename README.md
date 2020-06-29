# 🐻 Remembear

[![Build Status][build-badge]][build-link] [![Code Coverage][coverage-badge]][coverage-link] [![Rust Crate][crates-badge]][crates-link] [![Documentation][docs-badge]][docs-link]

Self-hosted web app for recurring reminders

Remembear was created to manage household chores but can be used for medication reminders, appointment notifications, and anything else occuring on a regular weekly or daily basis!

## Installation

Remembear is still under active development but will be available as a binary and Docker container once released. Until then, you can check out the [contributor guidelines](https://github.com/codehearts/remembear/blob/master/CONTRIBUTING.md) for steps to build and run remembear locally

## Usage

### CLI Usage

- User
  - Add: Adds a new user (`remembear user add <name>`)
  - List: Lists all users as a JSON array (`remembear user list`)
  - Update: Updates an existing user (`remembear user update <uid> [-n name]`)
  - Remove: Removes a user by their uid (`remembear user remove <uid>`)

## Development

If you'd like to contribute to remembear's development, [CONTRIBUTING.md](https://github.com/codehearts/remembear/blob/master/CONTRIBUTING.md) will get you started. You can also [open an issue](https://github.com/codehearts/remembear/issues/new) for any bugs or feature requests, that's just as valuable as code contributions!

[build-badge]:    https://img.shields.io/github/workflow/status/codehearts/remembear/Build/master?logo=github&logoColor=white
[build-link]:     https://github.com/codehearts/remembear/actions?query=workflow%3ABuild+branch%3Amaster
[coverage-badge]: https://img.shields.io/codecov/c/github/codehearts/remembear?logo=codecov&logoColor=white
[coverage-link]:  https://codecov.io/gh/codehearts/remembear
[crates-badge]:   https://img.shields.io/crates/v/remembear?logo=rust&logoColor=white
[crates-link]:    https://crates.io/crates/remembear
[docs-badge]:     https://docs.rs/remembear/badge.svg
[docs-link]:      https://docs.rs/remembear
