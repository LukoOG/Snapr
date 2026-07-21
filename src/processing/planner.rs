use std::{fs, path::Path};

use crate::{
    error::SnaprResult,
    models::{
        EntryBuildResult, FileEntry, FileProcessResult, FileStoreReport,
        workspace_index::{IndexLookup, IndexedFile, WorkspaceIndex},
    },
    processing::{file::process_file, hash::hash_file_chunks},
};

pub enum EntryMode {
    Hash,
    Store,
}

impl EntryMode {
    pub fn process(self, path: &Path) -> SnaprResult<FileProcessResult> {
        match self {
            EntryMode::Hash => hash_file_chunks(path),
            EntryMode::Store => process_file(path),
        }
    }
}

pub fn build_entry(
    path: &Path,
    mode: EntryMode,
    index: &WorkspaceIndex,
) -> SnaprResult<EntryBuildResult> {
    let metadata = fs::metadata(path)?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path_string = path.to_string_lossy().into_owned();

    match index.lookup(path.to_string_lossy().as_ref()) {
        IndexLookup::Hit(cached) if cached.matches(&metadata) => {
            let entry = FileEntry::build(
                path_string.clone(),
                cached.chunk_hashes.clone(),
            );
            
            return Ok(EntryBuildResult {
                path: path_string.clone(),
                entry,
                report: FileStoreReport::default(),
                cache_entry: cached.clone(),
            });
        }

        _ => {
            let result = mode.process(path)?;

            let entry = FileEntry::build(
                path_string.clone(),
                result.chunk_hashes.clone(),
            );

            let cache_entry = IndexedFile::new(modified, metadata.len(), result.chunk_hashes);

            Ok(EntryBuildResult {
                path: result.path.to_string_lossy().to_string(),
                entry,
                report: result.report,
                cache_entry,
            })
        }
    }
}
