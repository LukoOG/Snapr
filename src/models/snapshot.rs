use serde::{Deserialize, Serialize};

use super::SnapshotStats;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub chunk_hashes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: u32,
    pub message: String,
    pub created_at: u64,
    pub files: Vec<FileEntry>,

    //metadata
    pub stats: SnapshotStats,
}

#[derive(Debug)]
pub struct WorkspaceSnapshot {
    pub files: Vec<FileEntry>,
}

pub trait SnapshotFiles {
    fn files(&self) -> &[FileEntry];
}

impl SnapshotFiles for Snapshot {
    fn files(&self) -> &[FileEntry] {
        &self.files
    }
}

impl WorkspaceSnapshot {
    pub fn build(entries: Vec<FileEntry>) -> Self {
        Self { files: entries }
    }
}

impl SnapshotFiles for WorkspaceSnapshot {
    fn files(&self) -> &[FileEntry] {
        &self.files
    }
}

impl FileEntry {
    pub fn build(path: String, hashes: Vec<String>) -> Self {
        FileEntry {
            path,
            chunk_hashes: hashes,
        }
    }
}
