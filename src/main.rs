use std::{env, error::Error};

mod cli;
mod commands;
mod models;
mod storage;
mod filesystem;

use cli::parse_args;
use commands::{Command, history::handle_history, init::handle_init, save::handle_save};
use storage::load_snapshots;
use filesystem::collect_files;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let command = parse_args(&args);

    match command {
        Command::Init => handle_init(),
        Command::History => {
            let snapshots = load_snapshots()?;
            handle_history(&snapshots)
        }
        Command::Save { message } => {
            let mut snapshots = load_snapshots()?;
            handle_save(&mut snapshots, message)
        }
    }?;
    let files = collect_files()?;
    for file in files {
        println!("{:?}", file)
    }
    Ok(())
}
