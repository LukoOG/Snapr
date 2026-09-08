use crate::{
    constants::SNAPSHOTS_FILE, error::SnaprResult, models::{Snapshot, SnapshotStats, WorkspaceStoreReport}, processing::build_snapshot_entries, storage::{load_config, save_config},
};
use std::{fs};

pub fn handle_save(snapshots: &mut Vec<Snapshot>, message: String) -> SnaprResult<WorkspaceStoreReport> {
    let mut config = load_config()?;

    let (entries, report) = build_snapshot_entries()?;
    let next_id = snapshots.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let mut stats = SnapshotStats::from(&report);
    stats.total_storage_bytes = config.total_storage_bytes + report.new_storage_bytes as u64;
    let new_snapshot = Snapshot {
        id: next_id,
        message,
        files: entries,
        created_at: chrono::Utc::now().timestamp() as u64,
        stats,
    };
    snapshots.push(new_snapshot);
    let json = serde_json::to_string_pretty(snapshots)?;
    fs::write(SNAPSHOTS_FILE, json)?;

    //config
    config.update_current_snapshot();
    config.update_total_storage_bytes(report.new_storage_bytes as u64);
    save_config(&config)?;
    Ok(report)
}
