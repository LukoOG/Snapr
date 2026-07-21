use std::{error::Error, fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{error::SnaprResult, models::{Chunk, HashedChunk}};

//trying std and hex crate
//split hashing and file loading responsibility later
pub fn hash_and_get_contents(path: &Path) -> Result<(String, Vec<u8>), Box<dyn Error>> {
    let mut hasher = Sha256::new();
    let contents = fs::read(path)?;
    hasher.update(&contents);
    let result = hasher.finalize();
    Ok((hex::encode(result), contents))
}

pub fn hash_chunk(chunk: Chunk) -> SnaprResult<HashedChunk> {
    let mut hasher = Sha256::new();
    hasher.update(&chunk.bytes);
    let result = hasher.finalize();
    let hashed_chunk = HashedChunk::from_chunk(chunk, hex::encode(result));
    Ok(hashed_chunk)
}