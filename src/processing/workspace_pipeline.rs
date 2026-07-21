use rayon::prelude::*;

use crate::error::SnaprResult;
use crate::filesystem::compress_chunk;
use crate::filesystem::{collect::collect_files, hash::hash_chunk};
use crate::models::{ChunkReader, FileEntry, FileProcessResult, FileStoreReport};
use crate::models::{DEFAULT_CHUNK_SIZE, WorkspaceStoreReport};
use crate::scoped_timer;
use crate::storage::store_chunk;
use std::{fs::File, path::Path};

fn hash_file_chunks(path: &Path) -> SnaprResult<Vec<String>> {
    let reader = File::open(path)?;
    let mut chunk_reader = ChunkReader::new(reader, DEFAULT_CHUNK_SIZE);
    let mut chunk_hashes = Vec::new();

    while let Some(chunk) = chunk_reader.next_chunk()? {
        let hashed = hash_chunk(chunk)?;
        chunk_hashes.push(hashed.hash);
    }

    Ok(chunk_hashes)
}

fn process_file(path: &Path) -> SnaprResult<FileProcessResult> {
    scoped_timer!("Process File: {path.display()} "); 
    let reader = File::open(path)?;
    let mut chunk_reader = ChunkReader::new(reader, DEFAULT_CHUNK_SIZE);
    let mut chunk_hashes = Vec::new();
    let mut report = FileStoreReport::default();

    while let Some(chunk) = chunk_reader.next_chunk()? {
        let hashed = hash_chunk(chunk)?;
        let compressed = compress_chunk(hashed)?;
        let stored = store_chunk(compressed)?;
        report.record(&stored);
        chunk_hashes.push(stored.hash);
    }

    Ok(FileProcessResult {
        path: path.to_path_buf(),
        chunk_hashes,
        report,
    })
}

pub fn build_entries() -> SnaprResult<Vec<FileEntry>> {
    let files = collect_files()?;
    let mut entries: Vec<FileEntry> = Vec::new();
    {
        for file in files {
            let hashes = hash_file_chunks(&file)?;

            entries.push(FileEntry::build(file.to_string_lossy().to_string(), hashes));
        }
    };
    Ok(entries)
}

pub fn build_snapshot_entries() -> SnaprResult<(Vec<FileEntry>, WorkspaceStoreReport)> {
    let mut entries: Vec<FileEntry> = Vec::new();
    let mut report = WorkspaceStoreReport::default();
    let results = collect_files()?
        .par_iter()
        .map(|file| process_file(&file))
        .collect::<SnaprResult<Vec<_>>>()?;
    {
        for result in results {
            report.merge(&result.report);
            entries.push(FileEntry::build(
                result.path.to_string_lossy().to_string(),
                result.chunk_hashes,
            ));
        }
    };
    Ok((entries, report))
}
