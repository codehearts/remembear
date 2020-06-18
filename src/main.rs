use remembear::initialize_dependencies;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let _dependencies = initialize_dependencies()?;

    Ok(())
}
