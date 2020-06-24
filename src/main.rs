use remembear::initialize_dependencies;
use std::error::Error;

use remembear::user::{self, model::NewUser};

fn main() -> Result<(), Box<dyn Error>> {
    let dependencies = initialize_dependencies()?;

    let user_provider = user::Provider::new(dependencies.database);

    user_provider.add(NewUser {
        name: String::from("leland"),
    })?;

    Ok(())
}
