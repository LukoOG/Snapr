use std::{env, error::Error};

mod cli;
mod commands;
mod models;
mod storage;
mod filesystem;
mod hash;

use cli::parse_args;
use commands::{Command, history::handle_history, init::handle_init, save::handle_save};
use storage::load_snapshots;
use filesystem::collect_files;
use hash::hash_file;

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

    //testing stuff
    let files = collect_files()?;
    for file in &files[0..7] {
        let (hash, _) = hash_file(file)?;
        println!("{:?}", file.display());
        println!("hash: {}\n", hash)
    }
    Ok(())
}
