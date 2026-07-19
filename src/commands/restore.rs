use super::helpers::calculate_diff;
use crate::config::{load_config, save_config};
use crate::filesystem::restore_file;
use crate::models::Snapshot;
use crate::processing::build_entries;


use std::collections::HashMap;
use std::{error::Error, fs};

pub fn handle_restore(snapshots: &[Snapshot], snapshot_id: u32) -> Result<(), Box<dyn Error>> {
    let mut config = load_config()?;

    //check if on current snapshot
    if config.current_snapshot == Some(snapshot_id) {
        println!("Already on snapshot {}", snapshot_id);
        return Ok(());
    }

    let target_snapshot = snapshots
        .iter()
        .find(|s| s.id == snapshot_id)
        .ok_or("Snapshot not found")?;
    let current_workspace = Snapshot::build_workspace(build_entries()?);

    let diff = calculate_diff(&current_workspace, target_snapshot);
    let target_map = target_snapshot
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.chunk_hashes.clone()))
        .collect::<HashMap<_, _>>();

    for path in &diff.removed {
        fs::remove_file(path)?;
    }

    for path in diff.added.iter().chain(diff.modified.iter()) {
        let hashes = target_map.get(path.as_str()).ok_or("Missing file in snapshot")?;
        restore_file(path, hashes)?;
    }

    let restored = diff.added.len() + diff.modified.len();
    let removed = diff.removed.len();
    let skipped = target_snapshot.files.len() - restored;

    println!("Restored snapshot {}\n", snapshot_id);
    println!("{} files restored", restored);
    println!("{} files skipped", skipped);
    println!("{} files removed", removed);

    //config
    config.set_current_snapshot(snapshot_id);
    save_config(&config)?;
    Ok(())
}
