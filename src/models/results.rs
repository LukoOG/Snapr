use crate::models::{FileEntry, workspace_index::IndexedFile};

use super::reports::FileStoreReport;

pub struct FileProcessResult {
    pub path: std::path::PathBuf,
    pub chunk_hashes: Vec<String>,
    pub report: FileStoreReport,
}
pub struct ChunkStoreResult {
    pub stored: bool,
    pub original_size: usize,
    pub compressed_size: usize,
    //To aid in builder pipeline
    pub hash: String,
}

pub struct EntryBuildResult {
    pub path: String,
    pub entry: FileEntry,
    pub report: FileStoreReport,
    pub cache_entry: IndexedFile,
}