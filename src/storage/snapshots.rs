use std::fs;

use crate::{constants::SNAPSHOTS_FILE, error::SnaprResult, models::Snapshot};

pub fn load_snapshots() -> SnaprResult<Vec<Snapshot>> {
    let contents = fs::read_to_string(SNAPSHOTS_FILE).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Snapr not initialized")
        } else {
            e
        }
    })?;
    let parsed = serde_json::from_str(&contents)?;
    Ok(parsed)
}