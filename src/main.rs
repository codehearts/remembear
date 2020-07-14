use remembear::command::{self, Command};
use remembear::{initialize_dependencies, reminder, user};
use std::error::Error;
use std::sync::Arc;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    let command = command::Global::from_args();
    let dependencies = initialize_dependencies()?;

    let user_provider = user::Provider::new(Arc::clone(&dependencies.database));
    let reminder_provider = reminder::Provider::new(Arc::clone(&dependencies.database));

    let providers = command::Providers {
        user: &user_provider,
        reminder: &reminder_provider,
    };

    match command.execute(providers) {
        Ok(output) => println!("{}", output),
        Err(error) => eprintln!("{}", error),
    };

    Ok(())
}
