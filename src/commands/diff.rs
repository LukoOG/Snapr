use crate::models::{FileEntry, Snapshot};
use std::{collections::HashMap, error::Error};

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

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    for (path, hash) in new_map.iter() {
        if let Some(old_hash) = old_map.get(path) {
            if hash != old_hash {
                modified.push(path.clone());
            }
        } else {
            added.push(path.clone())
        }
    }

    for (path, _) in old_map.iter() {
        if !new_map.contains_key(path) {
            removed.push(path.clone())
        }
    }

    println!("Added files: {:?}", added);
    println!("Modified files: {:?}", modified);
    println!("Removed files: {:?}", removed);

    Ok(())
}
