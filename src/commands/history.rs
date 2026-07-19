use std::error::Error;

use crate::{config::load_config, models::Snapshot};

pub fn handle_history(snapshots: &[Snapshot]) -> Result<(), Box<dyn Error>> {
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
