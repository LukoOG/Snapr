use crate::{
    constants::{FLAG_NONE, HEADER_SIZE, MAGIC, VERSION_1}, error::SnaprResult, models::{CompressedChunk, CompressionType, HashedChunk},
};
use zstd::{encode_all};

pub fn compress_chunk(chunk: HashedChunk) -> SnaprResult<CompressedChunk> {
    if chunk.hash.len() != 64 {
        return Err("invalid sha256 hash".into());
    }
    let compressed = encode_all(&chunk.bytes[..], 3)?;
    let mut object = Vec::<u8>::with_capacity(HEADER_SIZE + compressed.len());
    object.extend_from_slice(MAGIC);
    object.push(VERSION_1);
    object.push(FLAG_NONE);
    object.push(CompressionType::Zstd as u8);
    object.extend_from_slice(&chunk.bytes.len().to_le_bytes());
    object.extend_from_slice(&compressed);

    Ok(CompressedChunk {
        index: chunk.index,
        hash: chunk.hash,
        object_bytes: object,
        original_size: chunk.bytes.len(),
    })
}
