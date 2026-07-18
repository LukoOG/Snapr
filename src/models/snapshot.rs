use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub object_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: u32,
    pub message: String,
    pub files: Vec<FileEntry>,
}

impl Snapshot {
    //Only the files field is actually needed
    pub fn build_workspace(entries: Vec<FileEntry>) -> Self {
        Self {
            id: 0,
            message: "current workspace".into(),
            files: entries,
        }
    }
}

impl FileEntry {
    pub fn build(path: String, object_hash: String) -> Self {
        FileEntry { path, object_hash }
    }
}
