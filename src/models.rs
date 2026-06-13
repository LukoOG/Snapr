use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    hash: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: u32,
    pub message: String,
    pub files: Vec<FileEntry>,
}