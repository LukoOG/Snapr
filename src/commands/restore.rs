use crate::config::{load_config, save_config};
use crate::filesystem::{build_entries, restore_file};
use crate::hash::hash_file_bytes;
use crate::models::{FileEntry, Snapshot};

use std::{error::Error, fs};

pub fn handle_restore(snapshots: &[Snapshot], snapshot_id: u32) -> Result<(), Box<dyn Error>> {
    let mut config = load_config()?;

    //check if on current snapshot
    if config.current_snapshot == Some(snapshot_id) {
        println!("Already on snapshot {}", snapshot_id);
        return Ok(());
    }

    let snapshot = snapshots
        .iter()
        .find(|s| s.id == snapshot_id)
        .ok_or("Snapshot not found")?;
    let current_workspace = Snapshot::build_workspace(build_entries()?);

    let mut restored = 0;
    let mut skipped = 0;
    let mut removed = 0;

    ///TODO: Move all this to using DiffResult
    for FileEntry { path, hash } in snapshot.files.iter() {
        let object_path = format!(".snapr/objects/{}", hash);
        if let Some(current) = fs::read(path).ok() {
            let current_hash = hash_file_bytes(&current)?;

            //file unchanged
            if &current_hash == hash {
                skipped += 1;
                continue;
            }

            //file modified
            restore_file(path, &object_path)?;
            restored += 1;
        } else {
            //file missing
            restore_file(path, &object_path)?;
            restored += 1;
        }
    }

    for entry in snapshot.files.iter() {
        if !current_workspace.files.contains(entry) {
            fs::remove_file(&entry.path)?;
            removed += 1;
        }
    }

    println!("Restored snapshot {}\n", snapshot_id);
    println!("{} files restored", restored);
    println!("{} files skipped", skipped);
    println!("{} files removed", removed);

    //config
    config.set_current_snapshot(snapshot_id);
    save_config(&config)?;
    Ok(())
}
