use crate::{commands::{helpers::compare_snapshots, models::DiffResult}, error::SnaprResult, models::Snapshot};

pub fn handle_diff(snapshots: &[Snapshot], old_id: u32, new_id: u32) -> SnaprResult<DiffResult> {
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
    
    Ok(compare_snapshots(old_snapshot, new_snapshot))
}
