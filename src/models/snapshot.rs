use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub hash: String,
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
    pub fn build(path: String, hash: String) -> Self {
        FileEntry { path, hash}
    }
}
