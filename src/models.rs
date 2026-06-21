use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub hash: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: u32,
    pub message: String,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnaprConfig {
    version: u32,
    pub current_snapshot: Option<u32>,
}

impl SnaprConfig {
    pub fn new() -> Self {
        SnaprConfig { version:1, current_snapshot: None }
    }
}
