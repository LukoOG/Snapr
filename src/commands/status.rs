use std::collections::HashMap;
use std::error::Error;

use crate::{
    commands::{helpers::calculate_diff, models::DiffResult},
    config::load_config,
    models::{FileEntry, Snapshot},
};

fn print_section(title: &str, symbol: char, entries: &[String]) {
    if entries.is_empty() {
        return;
    }

    println!("\n {}", title);

    for entry in entries {
        println!("{} {}", symbol, entry)
    }
}

fn print_status(diff: &DiffResult) {
    println!("Current Workspace Status");
    println!("\nSummary");
    println!(
        "{} added, {} modified, {} removed",
        diff.added.len(),
        diff.modified.len(),
        diff.removed.len()
    );
    print_section("Added", '+', &diff.added);
    print_section("Modified", '~', &diff.modified);
    print_section("Removed", '-', &diff.removed);
}

// pub fn handle_status<V: AsRef<Vec<FileEntry>> + Iterator>(snapshots: &[Snapshot], entries: V) -> Result<(), Box<dyn Error>> {
pub fn handle_status(
    snapshots: &[Snapshot],
    entries: Vec<FileEntry>,
) -> Result<(), Box<dyn Error>> {
    let config = load_config()?;
    if let None = config.current_snapshot {
        eprintln!("No Snapshots added");
        return Ok(());
    }
    //compare current snapshot and current workspace state
    let snapshot = snapshots
        .iter()
        .find(|s| s.id == config.get_current_snapshot())
        .ok_or("Snapshot not found!")?;

    let current_workspace_snapshot = Snapshot::build_workspace(entries);

    let result = calculate_diff(&snapshot, &current_workspace_snapshot);

    print_status(&result);
    Ok(())
}
