use crate::config::{load_config, save_config};
use crate::filesystem::restore_file;
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
    let mut restored = 0;
    let mut skipped = 0;

    for FileEntry { path, hash } in snapshot.files.iter() {
        let object_path = format!(".snapr/objects/{}", hash);
        if let Some(current) = fs::read(path).ok() {
            let current_hash = hash_file_bytes(&current)?;

            if &current_hash == hash {
                skipped += 1;
                continue;
            }

            restore_file(path, &object_path)?;
            restored += 1;
        } else {
            restore_file(path, &object_path)?;
            restored += 1;
        }
    }

    println!("Restored snapshot {}\n", snapshot_id);
    println!("{} files restored", restored);
    println!("{} files skipped", skipped);
    // println!("{} files deleted", deleted);

    //config
    config.set_current_snapshot(snapshot_id);
    save_config(&config)?;
    Ok(())
}
