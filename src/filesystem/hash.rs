use sha2::{Digest, Sha256};

use crate::{
    error::SnaprResult, models::{Chunk, HashedChunk},
};

pub fn hash_chunk(chunk: Chunk) -> SnaprResult<HashedChunk> {
    let mut hasher = Sha256::new();
    hasher.update(&chunk.bytes);
    let result = hasher.finalize();
    let hashed_chunk = HashedChunk::from_chunk(chunk, hex::encode(result));
    Ok(hashed_chunk)
}

pub fn hash_chunk_bytes(bytes: &[u8]) -> SnaprResult<String> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}
