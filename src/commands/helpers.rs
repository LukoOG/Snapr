use std::collections::HashMap;

use super::models::DiffResult;
use crate::models::{FileEntry, Snapshot};

fn print_section(title: &str, symbol: char, entries: &[String]) {
    if entries.is_empty() {
        return;
    }

    println!("\n {}", title);

    for entry in entries {
        println!("{} {}", symbol, entry)
    }
}

fn print_diff(diff: &DiffResult) {
    if diff.added.is_empty() && diff.modified.is_empty() && diff.removed.is_empty() {
        println!("Workspace is clean!");
        return;
    }
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

pub(super) fn calculate_diff(old_snapshot: &Snapshot, new_snapshot: &Snapshot) -> DiffResult {
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

    result.added.sort();
    result.modified.sort();
    result.removed.sort();

    result
}

pub(super) fn compare_snapshots(left: &Snapshot, right: &Snapshot, title: &str) {
    let diff = calculate_diff(left, right);
    println!("{}", title);
    print_diff(&diff);
}
