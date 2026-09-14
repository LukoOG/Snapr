use std::fs;
use std::path::Path;

use crate::{constants::{CONFIG_FILE, OBJECTS_DIR, SNAPSHOTS_FILE, WORKSPACE_INDEX_FILE}, error::SnaprResult};

pub fn handle_init() -> SnaprResult<()> {
    let path = Path::new(".snapr");

    if path.exists() {
        return Err("Snapr already initialized!".into());
    }

    fs::create_dir_all(OBJECTS_DIR)?;
    fs::write(
        CONFIG_FILE,
        r#"{"version": 1,"total_storage_bytes":0}"#,
    )?;
    fs::write(SNAPSHOTS_FILE, "[]")?;
    fs::write(WORKSPACE_INDEX_FILE, r#"{"files":{}}"#)?;

    Ok(())
}
