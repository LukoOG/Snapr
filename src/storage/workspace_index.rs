use std::{fs, todo};

use crate::{constants::WORKSPACE_INDEX_FILE, error::SnaprResult, models::workspace_index::WorkspaceIndex};

pub fn load_workspace_index() -> SnaprResult<WorkspaceIndex> {
    let contents = fs::read_to_string(WORKSPACE_INDEX_FILE).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Snapr not initialized")
        } else {
            e
        }
    })?;
    let parsed = serde_json::from_str(&contents)?;
    Ok(parsed)
}

pub fn save_workspace_index(index: &WorkspaceIndex) -> SnaprResult<()> {
    let json = serde_json::to_string_pretty(index)?;
    fs::write(WORKSPACE_INDEX_FILE, json)?;
    Ok(())
}

pub fn reset_workspace_index() -> SnaprResult<()> {
    let default = serde_json::to_string_pretty(&WorkspaceIndex::default())?;
    fs::write(WORKSPACE_INDEX_FILE, default)?;
    Ok(())
}

pub fn rebuild_workspace_index() -> SnaprResult<WorkspaceIndex> {
    todo!()
}
