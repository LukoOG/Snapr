use std::{env, error::Error};

mod cli;
mod commands;
mod models;
mod storage;

use cli::parse_args;
use commands::{Command, history::handle_history, init::handle_init, save::handle_save};
use storage::load_snapshots;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let command = parse_args(&args);

    let results = match command {
        Command::Init => handle_init(),
        Command::History => {
            let snapshots = load_snapshots()?;
            handle_history(&snapshots)
        },
        // Command::Save { message } => handle_save(message),
        _ => Ok(()),
    }?;
    Ok(())
}
