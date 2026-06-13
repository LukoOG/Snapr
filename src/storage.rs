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
    let contents = fs::read_to_string(".snapr/objects/snapshots.json");
    let parsed = match contents {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(parsed) => parsed,
            Err(e) => return Err(Box::new(e)),
        },
        Err(e) => {
            eprintln!("snapr workspace not initialized: {e}");
            return Err(Box::new(e));
        }
    };

    Ok(parsed)
}
