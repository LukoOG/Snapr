use crate::models::Snapshot;
use std::{error::Error, fs};

/// Loads the snapshots from a snapr workspace
///
/// # Returns
///
/// - `Vec<Snapshot>` - Returns a vector of the snapshots if present. Returns empty vector is anything fails silently
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let _ = load_snapshots();
/// ```
pub fn load_snapshots() -> Result<Vec<Snapshot>, Box<dyn Error>> {
    let contents = fs::read_to_string(".snapr/snapshots.json").map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Snapr not initialized")
        } else {
            e
        }
    })?;
    let parsed = serde_json::from_str(&contents)?;
    Ok(parsed)
}
