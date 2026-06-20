use crate::hash::{hash_file, hash_file_bytes};
use crate::models::FileEntry;
use crate::models::Snapshot;

use std::error::Error;
use std::fs;
use std::path::Path;

fn restore_file(path: &str, object_path: &str) -> Result<(), Box<dyn Error>> {
    let contents = fs::read(object_path).map_err(|_| format!("Missing object: {}", object_path))?;

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;
    Ok(())
}

pub fn handle_restore(snapshots: &[Snapshot], snapshot_id: u32) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}
