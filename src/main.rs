use std::{env, error::Error};

mod cli;
mod commands;
mod config;
mod workspace;
mod hash;
mod models;
mod storage;
mod constants;

use cli::parse_args;
use commands::{
    Command, diff::handle_diff, history::handle_history, init::handle_init, save::handle_save, restore::handle_restore, status::handle_status
};
use workspace::build_entries;
use storage::load_snapshots;

use crate::{commands::save::print_save_report, workspace::build_and_store_entries};

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
            let (entries, report) = build_and_store_entries()?;
            print_save_report(0, &message, &report);
            handle_save(&mut snapshots, message, entries)
        }
        Command::Diff(old, new) => {
            let snapshots = load_snapshots()?;
            handle_diff(&snapshots, old, new)
        }
        Command::Restore(snapshot_id) => {
            let snapshots = load_snapshots()?;
            handle_restore(&snapshots, snapshot_id)
        }
        Command::Status => {
            let snapshots = load_snapshots()?;
            let entries = build_entries()?; //current workspace entries
            handle_status(&snapshots, entries)
        }
    }?;
    Ok(())
}