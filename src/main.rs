use std::{env, error::Error};

mod cli;
mod commands;
mod config;
mod error;
mod models;
mod storage;
mod filesystem;
mod constants;
mod processing;
mod ui;

use cli::parse_args;
use commands::{
    Command, diff::handle_diff, history::handle_history, init::handle_init, save::handle_save, restore::handle_restore, status::handle_status
};
use processing::{build_entries};
use storage::load_snapshots;
use error::SnaprResult;

fn main() -> SnaprResult<()> {
    let args: Vec<String> = env::args().collect();

    let command = parse_args(&args);

    let _ = match command {
        Command::Init => handle_init(),
        Command::History => {
            let snapshots = load_snapshots()?;
            handle_history(&snapshots)?;
            Ok(())
        }
        Command::Save { message } => {
            let mut snapshots = load_snapshots()?;
            let report = handle_save(&mut snapshots, message)?;
            ui::print_save_report(snapshots.last().unwrap().id, &snapshots.last().unwrap().message, &report);
            Ok(())
        }
        Command::Diff(old, new) => {
            let snapshots = load_snapshots()?;
            handle_diff(&snapshots, old, new)?;
            Ok(())
        }
        Command::Restore(restore_options) => {
            let snapshots = load_snapshots()?;
            let report = handle_restore(&snapshots, restore_options)?;
            if report.dry_run {
                ui::print_restore_dry_run_report(snapshots.last().unwrap().id, &report);
            } else if !report.dry_run {
                ui::print_restore_report(&report);
            }
            Ok(())
        }
        Command::Status => {
            let snapshots = load_snapshots()?;
            let entries = build_entries()?; //current workspace entries
            handle_status(&snapshots, entries)?;
            Ok(())
        }
    };
    Ok(())
}