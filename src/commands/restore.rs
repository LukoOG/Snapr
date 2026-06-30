use super::helpers::calculate_diff;
use crate::config::{load_config, save_config};
use crate::workspace::build_entries;
use crate::storage::restore_object;
use crate::models::{Snapshot};

use std::collections::{HashMap};
use std::{error::Error, fs};

pub fn handle_restore(snapshots: &[Snapshot], snapshot_id: u32) -> Result<(), Box<dyn Error>> {
    let mut config = load_config()?;

    //check if on current snapshot
    if config.current_snapshot == Some(snapshot_id) {
        println!("Already on snapshot {}", snapshot_id);
        return Ok(());
    }

    let target_snapshot = snapshots.iter().find(|s| s.id == snapshot_id).ok_or("Snapshot not found")?;
    let current_workspace = Snapshot::build_workspace(build_entries()?);

    let diff = calculate_diff(&current_workspace, target_snapshot);
    let target_map = target_snapshot.files.iter().map(|f| (f.path.as_str(), f.hash.as_str())).collect::<HashMap<_, _>>();


    for path in &diff.removed {
        fs::remove_file(path)?;
    }

    for path in diff.added.iter().chain(diff.modified.iter()) {
        let hash = target_map.get(path.as_str()).ok_or("Missing file in shapshot")?;
        let object_path = format!(".snapr/objects/{}", *hash);
        restore_object(path, &object_path)?;
    }

    let restored = diff.added.len() + diff.modified.len();
    let removed = diff.removed.len();
    let skipped = target_snapshot.files.len() - restored;

    //TODO: Move all this to using DiffResult
    // for FileEntry { path, hash } in snapshot.files.iter() {
    //     let object_path = format!(".snapr/objects/{}", hash);
    //     if let Some(current) = fs::read(path).ok() {
    //         let current_hash = hash_file_bytes(&current)?;

    //         //file unchanged
    //         if &current_hash == hash {
    //             skipped += 1;
    //             continue;
    //         }

    //         //file modified
    //         restore_file(path, &object_path)?;
    //         restored += 1;
    //     } else {
    //         //file missing
    //         restore_file(path, &object_path)?;
    //         restored += 1;
    //     }
    // }

    // let current_paths = current_workspace.files.iter().map(|f| f.path.as_str()).collect::<HashSet<&str>>();

    // for entry in current_paths {
    //     if !snapshot.files.iter().any(|f| f.path == entry) {
    //         fs::remove_file(entry)?;
    //         removed += 1;
    //     }
    // }

    println!("Restored snapshot {}\n", snapshot_id);
    println!("{} files restored", restored);
    println!("{} files skipped", skipped);
    println!("{} files removed", removed);

    //config
    config.set_current_snapshot(snapshot_id);
    save_config(&config)?;
    Ok(())
}
