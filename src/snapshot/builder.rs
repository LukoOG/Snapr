use crate::filesystem::compress_chunk;
use crate::filesystem::{collect::collect_files, hash::hash_chunk};
use crate::models::{Chunk, ChunkReader};
use crate::models::{DEFAULT_CHUNK_SIZE, FileEntry, StoreReport};
use crate::storage::store_object;
use std::{error::Error, fs::File, path::Path};

fn hash_file_chunks(path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let reader = File::open(path)?;
    let mut chunk_reader = ChunkReader::new(reader, DEFAULT_CHUNK_SIZE);
    let mut file_hashes = Vec::new();

    while let Some(chunk) = chunk_reader.next_chunk()? {
        let hash = hash_chunk(&chunk)?;
        file_hashes.push(hash);
    }

    Ok(file_hashes)
}

fn store_file_chunks(path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let reader = File::open(path)?;
    let mut chunk_reader = ChunkReader::new(reader, DEFAULT_CHUNK_SIZE);
    let mut report = StoreReport::default();
    let mut file_hashes = Vec::new();

    while let Some(chunk) = chunk_reader.next_chunk()? {
        let hashed = hash_chunk(chunk)?;
        let compressed = compress_chunk(hashed)?;
        let stored = store_object(compressed)?;
        file_hashes.push(stored.hash);
    }

    Ok(file_hashes)
}

pub fn build_entries() -> Result<Vec<FileEntry>, Box<dyn Error>> {
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

pub fn build_snapshot_entries() -> Result<Vec<FileEntry>, Box<dyn Error>> {
    let files = collect_files()?;
    let mut entries: Vec<FileEntry> = Vec::new();
    {
        for file in files {
            let hashes = store_file_chunks(&file)?;

            entries.push(FileEntry::build(file.to_string_lossy().to_string(), hashes));
        }
    };
    Ok(entries)
}
