use std::{fs, error::Error};
use crate::models::{FileEntry, Snapshot};

pub fn handle_save(snapshots: &mut Vec<Snapshot>, message: String, entries: Vec<FileEntry>) -> Result<(), Box<dyn Error>>{
    let next_id =  snapshots.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let new_snapshot = Snapshot {
        id: next_id,
        message,
        files: entries,
    };
    snapshots.push(new_snapshot);
    let json = serde_json::to_string_pretty(snapshots)?;
    fs::write(".snapr/snapshots.json", json)?;
    Ok(())
}