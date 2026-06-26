use serde::{Deserialize, Serialize};
#[repr(u8)]
pub enum CompressionType {
    None = 0,
    Zstd = 1,
}

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

pub struct StoreResult {

}

impl Snapshot {
    //Only the files field is actually needed
    pub fn build_workspace(entries: Vec<FileEntry>) -> Self {
        Self {
            id: 0,
            message: "current workspace".into(),
            files: entries
        }
    }
}

impl SnaprConfig {
    pub fn new() -> Self {
        SnaprConfig {
            version: 1,
            current_snapshot: None,
        }
    }

    // pub fn get_current_snapshot(&self) -> u32 {
    //     self.current_snapshot.unwrap()
    // }

    pub fn update_current_snapshot(&mut self) {
        match self.current_snapshot {
            None => self.current_snapshot = Some(1),
            Some(id) => self.current_snapshot = Some(1 + id),
        }
    }

    pub fn set_current_snapshot(&mut self, id: u32) {
        self.current_snapshot = Some(id)
    }
}
