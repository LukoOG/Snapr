use std::{fs::File, path::Path};

use crate::{
    error::SnaprResult, filesystem::{compress_chunk, hash::hash_chunk}, models::{ChunkReader, DEFAULT_CHUNK_SIZE, FileProcessResult, FileStoreReport}, scoped_timer, storage::store_chunk,
};

pub fn process_file(path: &Path) -> SnaprResult<FileProcessResult> {
    scoped_timer!(
        "Process File: {}",
        path.file_name().unwrap().to_string_lossy()
    );
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
