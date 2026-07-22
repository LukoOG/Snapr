use std::error::Error;
use std::fs;
use std::path::Path;

use crate::constants::{CONFIG_FILE, OBJECTS_DIR, SNAPSHOTS_FILE, WORKSPACE_INDEX_FILE};

pub fn handle_init() -> Result<(), Box<dyn Error>> {
    let path = Path::new(".snapr");

    if path.exists() {
        println!("Snapr already initialized!");
        return Ok(());
    }

    fs::create_dir_all(OBJECTS_DIR)?;
    fs::write(
        CONFIG_FILE,
        r#"{"version": 1,"repository_size":0}"#,
    )?;
    fs::write(SNAPSHOTS_FILE, "[]")?;
    fs::write(WORKSPACE_INDEX_FILE, r#"{"files":{}}"#)?;
    println!("Initialized snapr workspace");

    Ok(())
}
