use std::{fs::File, path::Path};

use crate::{error::SnaprResult, filesystem::hash::hash_chunk, models::{ChunkReader, DEFAULT_CHUNK_SIZE, FileProcessResult, FileStoreReport}};



pub fn hash_file_chunks(path: &Path) -> SnaprResult<FileProcessResult> {
    let reader = File::open(path)?;
    let mut chunk_reader = ChunkReader::new(reader, DEFAULT_CHUNK_SIZE);
    let mut chunk_hashes = Vec::new();

    while let Some(chunk) = chunk_reader.next_chunk()? {
        let hashed = hash_chunk(chunk)?;
        chunk_hashes.push(hashed.hash);
    }

    Ok(FileProcessResult {
        path: path.to_path_buf(),
        chunk_hashes,
        report: FileStoreReport::default(),
    })
}