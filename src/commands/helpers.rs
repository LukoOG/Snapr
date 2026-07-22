use std::collections::HashMap;

use super::models::DiffResult;
use crate::{models::{FileEntry, SnapshotFiles}, scoped_timer};

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

pub(super) fn calculate_diff<F: SnapshotFiles, T: SnapshotFiles>(from: &F, to: &T) -> DiffResult {
    let source = from
        .files()
        .iter()
        .map(|FileEntry { chunk_hashes, path }| (path.as_str(), chunk_hashes.as_slice()))
        .collect::<HashMap<_, _>>();
    let target = to
        .files()
        .iter()
        .map(|FileEntry { chunk_hashes, path }| (path.as_str(), chunk_hashes.as_slice()))
        .collect::<HashMap<_, _>>();

    let mut result = DiffResult::default();

    for (path, hash) in target.iter() {
        if let Some(old_hash) = source.get(path) {
            if hash != old_hash {
                result.modified.push((*path).to_owned());
            }
        } else {
            result.added.push((*path).to_owned())
        }
    }

    for path in source.keys() {
        if !target.contains_key(path) {
            result.removed.push((*path).to_owned())
        }
    }

    result.added.sort();
    result.modified.sort();
    result.removed.sort();

    result
}

pub(super) fn compare_snapshots<F: SnapshotFiles, T: SnapshotFiles>(from: &F, to: &T, title: &str) {
    scoped_timer!("Comparing");
    let diff = calculate_diff(from, to);
    println!("{}", title);
    print_diff(&diff);
}
