// pub workspace_bytes: u64,
// pub repository_bytes: u64,
// pub chunk_count: u64,

use serde::{Deserialize, Serialize};

use crate::models::WorkspaceStoreReport;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SnapshotStats {
    pub file_count: usize,
    pub chunk_count: usize,

    pub workspace_bytes: u64,

    pub new_chunks: usize,
    pub reused_chunks: usize,
    pub new_storage_bytes: u64,
    pub total_storage_bytes: u64,
}

impl From<&WorkspaceStoreReport> for SnapshotStats {
    fn from(report: &WorkspaceStoreReport) -> Self {
        SnapshotStats {
            file_count: report.total_files,
            chunk_count: report.total_chunks,
            workspace_bytes: report.workspace_bytes as u64,
            new_chunks: report.new_chunks,
            reused_chunks: report.reused_chunks,
            new_storage_bytes: report.new_storage_bytes as u64,
            total_storage_bytes: 0, // This will be updated later in the save command
        }
    }
}
