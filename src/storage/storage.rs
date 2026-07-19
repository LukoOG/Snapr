use crate::{
    constants::{HEADER_SIZE, MAGIC, OBJECTS_DIR},
    models::{ChunkStoreResult, CompressedChunk, Snapshot},
};
use std::{error::Error, fs, path::Path};
use zstd::decode_all;

pub fn load_snapshots() -> Result<Vec<Snapshot>, Box<dyn Error>> {
    let contents = fs::read_to_string(".snapr/snapshots.json").map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Snapr not initialized")
        } else {
            e
        }
    })?;
    let parsed = serde_json::from_str(&contents)?;
    Ok(parsed)
}

pub fn store_chunk(chunk: CompressedChunk) -> Result<ChunkStoreResult, Box<dyn Error>> {
    let path = format!("{}/{}", OBJECTS_DIR, chunk.hash);

    if Path::new(&path).exists() {
        return Ok(ChunkStoreResult {
            hash: chunk.hash,
            stored: false,
            original_size: chunk.original_size,
            compressed_size: chunk.object_bytes.len(),
        });
    }

    fs::write(path, &chunk.object_bytes)?;

    Ok(ChunkStoreResult {
        hash: chunk.hash,
        stored: true,
        original_size: chunk.original_size,
        compressed_size: chunk.object_bytes.len(),
    })
}

pub fn restore_chunk(hash: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let object_path = format!("{}/{}", OBJECTS_DIR, hash);
    let object = fs::read(&object_path).map_err(|_| format!("Missing object: {}", object_path))?;
    if &object[..5] != MAGIC {
        return Err("invalid object".into());
    }

    let _version = object[5];
    let _flags = object[6];
    let _compression = object[7];

    let _original_size = u64::from_le_bytes(object[8..16].try_into()?);

    let compressed = &object[HEADER_SIZE..];
    let contents = decode_all(&compressed[..])?;

    Ok(contents)
}
