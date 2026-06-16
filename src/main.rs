use std::{env, error::Error};

mod cli;
mod commands;
mod filesystem;
mod hash;
mod models;
mod storage;

use cli::parse_args;
use commands::{Command, history::handle_history, init::handle_init, save::handle_save};
use storage::load_snapshots;

use crate::filesystem::build_entries;


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
            let entries = build_entries()?;
            handle_save(&mut snapshots, message, entries)
        }
        Command::Diff(old, new) => {
            todo!()
        }
    }?;

    //testing stuff
    // let files = collect_files()?;
    // for file in &files[0..7] {
    //     let (hash, _) = hash_file(file)?;
    //     println!("{:?}", file.display());
    //     println!("hash: {}\n", hash)
    // }
    Ok(())
}
