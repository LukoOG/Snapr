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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::hash::hash_chunk;
    use crate::models::Chunk;

    fn make_hashed(bytes: Vec<u8>) -> crate::models::HashedChunk {
        hash_chunk(Chunk::new(0, bytes)).unwrap()
    }

    // Invariant 9: full round-trip — compress → read back → matches original bytes.
    #[test]
    fn compress_decompress_roundtrip() {
        let original = b"round trip test data for snapr compression".to_vec();
        let hashed = make_hashed(original.clone());
        let compressed = compress_chunk(hashed).unwrap();

        // Decompress the payload (skip 16-byte header).
        let decompressed = zstd::decode_all(&compressed.object_bytes[16..]).unwrap();
        assert_eq!(decompressed, original);
    }

    // Invariant 10: compress_chunk rejects a HashedChunk with a non-64-char hash.
    #[test]
    fn compress_chunk_rejects_short_hash() {
        let hashed = crate::models::HashedChunk {
            index: 0,
            hash: "tooshort".to_string(),
            bytes: b"some bytes".to_vec(),
        };
        let result = compress_chunk(hashed);
        assert!(result.is_err());
    }

    // Object starts with the SNAPR magic bytes.
    #[test]
    fn compressed_object_has_correct_magic() {
        let hashed = make_hashed(b"magic check".to_vec());
        let compressed = compress_chunk(hashed).unwrap();
        assert_eq!(&compressed.object_bytes[..5], b"SNAPR");
    }

    // Version byte is 1.
    #[test]
    fn compressed_object_version_is_1() {
        let hashed = make_hashed(b"version check".to_vec());
        let compressed = compress_chunk(hashed).unwrap();
        assert_eq!(compressed.object_bytes[5], 1); // VERSION_1
    }

    // Compression byte is 1 (Zstd).
    #[test]
    fn compressed_object_compression_is_zstd() {
        let hashed = make_hashed(b"compression type check".to_vec());
        let compressed = compress_chunk(hashed).unwrap();
        assert_eq!(compressed.object_bytes[7], 1); // CompressionType::Zstd
    }

    // The original_size field in header matches the actual original byte count.
    #[test]
    fn compressed_object_header_original_size_matches() {
        let original = b"size field check".to_vec();
        let len = original.len() as u64;
        let hashed = make_hashed(original);
        let compressed = compress_chunk(hashed).unwrap();

        let stored_size = u64::from_le_bytes(compressed.object_bytes[8..16].try_into().unwrap());
        assert_eq!(stored_size, len);
    }

    // original_size field on CompressedChunk also matches.
    #[test]
    fn compressed_chunk_original_size_field_matches_bytes() {
        let original = b"original_size consistency".to_vec();
        let expected_len = original.len();
        let hashed = make_hashed(original);
        let compressed = compress_chunk(hashed).unwrap();
        assert_eq!(compressed.original_size, expected_len);
    }

    // Round-trip for empty bytes.
    #[test]
    fn compress_decompress_empty_bytes() {
        let hashed = make_hashed(vec![]);
        let compressed = compress_chunk(hashed).unwrap();
        let decompressed = zstd::decode_all(&compressed.object_bytes[16..]).unwrap();
        assert_eq!(decompressed, b"");
    }

    // object_bytes total length is always >= HEADER_SIZE (16).
    #[test]
    fn compressed_object_at_least_header_size() {
        let hashed = make_hashed(b"min size".to_vec());
        let compressed = compress_chunk(hashed).unwrap();
        assert!(compressed.object_bytes.len() >= 16);
    }
}
