use crate::models::{FileEntry, Snapshot};
use std::{collections::HashMap, error::Error};

use super::models::DiffResult;

fn calculate_diff(old_snapshot: &Snapshot, new_snapshot: &Snapshot) -> DiffResult {
    let old_map = old_snapshot
        .files
        .iter()
        .map(|FileEntry { hash, path }| (path.clone(), hash.clone()))
        .collect::<HashMap<String, String>>();
    let new_map = new_snapshot
        .files
        .iter()
        .map(|FileEntry { hash, path }| (path.clone(), hash.clone()))
        .collect::<HashMap<String, String>>();

    let mut result = DiffResult::default();

    for (path, hash) in new_map.iter() {
        if let Some(old_hash) = old_map.get(path) {
            if hash != old_hash {
                result.modified.push(path.clone());
            }
        } else {
            result.added.push(path.clone())
        }
    }

    for path in old_map.keys() {
        if !new_map.contains_key(path) {
            result.removed.push(path.clone())
        }
    }

    result
}

fn print_section(title: &str, symbol: char, entries: &[String]){
    if entries.is_empty() {
        return;
    }

    println!("\n {}", title);

    for entry in entries {
        println!("{} {}", symbol, entry)
    }
}

fn print_diff(old_id: u32, new_id: u32, diff: &DiffResult) {
    println!("Comparing Snapshot {} -> {}", old_id, new_id);
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

pub fn handle_diff(snapshots: &[Snapshot], old_id: u32, new_id: u32) -> Result<(), Box<dyn Error>> {
    if old_id == 0 || new_id == 0 {
        return Err("Snapshot ids start at 1".into());
    }

    let old_snapshot = snapshots
        .iter()
        .find(|s| s.id == old_id)
        .ok_or("Old snapshot not found")?;
    let new_snapshot = snapshots
        .iter()
        .find(|s| s.id == new_id)
        .ok_or("New snapshot not found")?;

    let result = calculate_diff(old_snapshot, new_snapshot);

    print_diff(old_id, new_id, &result);

    Ok(())
}
