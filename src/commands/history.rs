use crate::{error::SnaprResult, models::Snapshot, storage::load_config};

pub fn handle_history(snapshots: &[Snapshot]) -> SnaprResult<u32> {
    let config = load_config()?;
    if snapshots.is_empty() {
        return Err("No snapshots yet!".into())
    }

    let current_id = match config.current_snapshot {
        Some(id) => id,
        None => {
            return Err("No current snapshot set.".into())
        }
    };
    Ok(current_id)
}
