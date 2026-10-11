use crate::{
    constants::{HEADER_SIZE, MAGIC, OBJECTS_DIR}, error::SnaprResult, models::{ChunkStoreResult, CompressedChunk},
};
use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
};
use zstd::decode_all;

pub fn store_chunk(chunk: CompressedChunk) -> SnaprResult<ChunkStoreResult> {
    let path = format!("{}/{}", OBJECTS_DIR, chunk.hash);

    // create_new makes "does it exist?" and "create it" a single atomic step,
    // so when parallel workers store the same chunk exactly one of them
    // creates the object and reports it as new.
    let stored = match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&chunk.object_bytes) {
                // Don't leave a partial object behind under a valid hash.
                let _ = fs::remove_file(&path);
                return Err(e.into());
            }
            true
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => false,
        Err(e) => return Err(e.into()),
    };

    Ok(ChunkStoreResult {
        hash: chunk.hash,
        stored,
        original_size: chunk.original_size,
        stored_size: chunk.object_bytes.len(),
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
    let contents = decode_all(compressed)?;

    Ok(contents)
}
