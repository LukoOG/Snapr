use crate::{
    config::{load_config, save_config}, models::{FileEntry, Snapshot, WorkspaceStoreReport}, processing::build_snapshot_entries,
};
use std::{error::Error, fs};

pub fn handle_save(snapshots: &mut Vec<Snapshot>, message: String) -> Result<WorkspaceStoreReport, Box<dyn Error>> {
    let mut config = load_config()?;

    let (entries, report) = build_snapshot_entries()?;
    let next_id = snapshots.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let new_snapshot = Snapshot {
        id: next_id,
        message,
        files: entries,
        chunk_count: report.total_chunks as u64,
        repository_bytes: report.new_storage_bytes as u64,
        workspace_bytes: report.original_bytes as u64,
    };
    snapshots.push(new_snapshot);
    let json = serde_json::to_string_pretty(snapshots)?;
    fs::write(".snapr/snapshots.json", json)?;

    //config
    config.update_current_snapshot();
    config.update_repository_size(report.new_storage_bytes as u64);
    save_config(&config)?;
    Ok(report)
}
