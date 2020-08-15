use remembear::{command, command::execute, reminder, user};
use remembear::{Config, Dependencies, Integrations, Providers};
use std::error::Error;
use std::sync::Arc;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let command = command::Global::from_args();

    let config = Config::load("remembear")?;
    let dependencies = Dependencies::new(&config)?;
    let integrations = Integrations::new(&config);

    let user_provider = user::Provider::new(Arc::clone(&dependencies.database));
    let reminder_provider = reminder::Provider::new(Arc::clone(&dependencies.database));

    let providers = Providers {
        user: &user_provider,
        reminder: &reminder_provider,
    };

    match execute(command, providers, integrations).await {
        Ok(output) => println!("{}", output),
        Err(error) => eprintln!("{}", error),
    };

    Ok(())
}
