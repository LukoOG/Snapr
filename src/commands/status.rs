use crate::{
    storage::load_config, error::SnaprResult, models::{FileEntry, Snapshot, WorkspaceSnapshot},
};
use super::{helpers::compare_snapshots};

// pub fn handle_status<V: AsRef<Vec<FileEntry>> + Iterator>(snapshots: &[Snapshot], entries: V) -> Result<(), Box<dyn Error>> {
pub fn handle_status(
    snapshots: &[Snapshot],
    entries: Vec<FileEntry>,
) -> SnaprResult<()> {
    let config = load_config()?;
    let workspace_id = match config.current_snapshot {
        Some(id) => id,
        None => {
            eprintln!("No snapshots yet");
            return Ok(())
        },
    };
    //compare current snapshot and current workspace state
    let snapshot = snapshots
        .iter()
        .find(|s| s.id == workspace_id)
        .ok_or("Snapshot not found!")?;

    let workspace = WorkspaceSnapshot::build(entries);

    compare_snapshots(snapshot, &workspace, "Current Workspace Status");
    Ok(())
}
