use crate::{
    constants::{HEADER_SIZE, MAGIC, OBJECTS_DIR}, error::SnaprResult, models::{ChunkStoreResult, CompressedChunk},
};
use std::{fs, path::Path};
use zstd::decode_all;

pub fn store_chunk(chunk: CompressedChunk) -> SnaprResult<ChunkStoreResult> {
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

pub fn read_chunk(hash: &str) -> SnaprResult<Vec<u8>> {
    let object_path = format!("{}/{}", OBJECTS_DIR, hash);
    let object = fs::read(&object_path).map_err(|_| format!("Missing object: {}", object_path))?;
    if object.len() < HEADER_SIZE {
        return Err("Object header is truncated".into());
    }
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
