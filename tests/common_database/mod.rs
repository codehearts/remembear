//! Shared database functionality between integration tests

use super::common::Result;
use remembear::database::{self, Database};
use std::io::sink;
use std::sync::Arc;

/// Creates a connection to a unique temporary in-memory sqlite3 database
#[inline]
pub fn new() -> Result<Arc<dyn Database>> {
    let database = database::Sqlite::connect(":memory:")?;

    diesel_migrations::run_pending_migrations_in_directory(
        database.connection(),
        &diesel_migrations::find_migrations_directory()?,
        &mut sink(),
    )?;

    Ok(Arc::new(database))
}
