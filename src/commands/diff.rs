use std::{collections::HashMap, error::Error};
use crate::models::Snapshot;

pub fn handle_diff(snapshots: &[Snapshot], old_id: u32, new_id: u32) -> Result<(), Box<dyn Error>> {
    let max = old_id.max(new_id);
    debug_assert!(snapshots.len() as u32 >= max, "Index out of Range");
    // let snap_map = HashMap<>::with_capacity(112);s
    for snapshot in snapshots.iter(){
        if snapshot.id == old_id || snapshot.id == new_id {
            println!("{:?}", snapshot)
        }
    }
    Ok(())
}