use crate::{config::load_config, error::SnaprResult, models::Snapshot};

pub fn handle_history(snapshots: &[Snapshot]) -> SnaprResult<()> {
    let config = load_config()?;
    if snapshots.is_empty() {
        println!("No snapshots yet!");
        return Ok(());
    }

    let current_id =    match config.current_snapshot {
        Some(id) => id,
        None => { 
            println!("No current snapshot set.");
            return Ok(());
        },
    };

    for snapshot in snapshots {
        if snapshot.id == current_id {
            println!("{}. {} {}", snapshot.id, snapshot.message, '*');
            continue;
        }
        println!("{}. {}", snapshot.id, snapshot.message)
    }

    Ok(())
}
