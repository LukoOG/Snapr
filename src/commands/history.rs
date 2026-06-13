use std::error::Error;

use crate::models::Snapshot;

pub fn handle_history(snapshots: &[Snapshot]) -> Result<(), Box<dyn Error>> {
    if snapshots.is_empty() {
        println!("No snapshots yet!");
        return Ok(())
    }

    for snapshot in snapshots {
        println!("{}. {}", snapshot.id, snapshot.message)
    }
    
    Ok(())
}
