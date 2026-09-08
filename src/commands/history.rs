use crate::{error::SnaprResult, models::Snapshot, storage::load_config, ui::print_history};

pub fn handle_history(snapshots: &[Snapshot]) -> SnaprResult<()> {
    let config = load_config()?;
    if snapshots.is_empty() {
        println!("No snapshots yet!");
        return Ok(());
    }

    let current_id = match config.current_snapshot {
        Some(id) => id,
        None => {
            println!("No current snapshot set.");
            return Ok(());
        }
    };
    print_history(snapshots, current_id);

    Ok(())
}
