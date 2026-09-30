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

#[cfg(test)]
mod tests {
    use super::*;

    // Hash is deterministic for the same bytes.
    #[test]
    fn hash_chunk_is_deterministic() {
        let bytes = b"hello snapr".to_vec();
        let chunk1 = crate::models::Chunk::new(0, bytes.clone());
        let chunk2 = crate::models::Chunk::new(0, bytes.clone());
        let h1 = hash_chunk(chunk1).unwrap();
        let h2 = hash_chunk(chunk2).unwrap();
        assert_eq!(h1.hash, h2.hash);
    }

    // Different bytes produce a different hash.
    #[test]
    fn different_bytes_produce_different_hash() {
        let c1 = crate::models::Chunk::new(0, b"aaa".to_vec());
        let c2 = crate::models::Chunk::new(0, b"bbb".to_vec());
        let h1 = hash_chunk(c1).unwrap().hash;
        let h2 = hash_chunk(c2).unwrap().hash;
        assert_ne!(h1, h2);
    }

    // Hash is exactly 64 hex characters (SHA-256).
    #[test]
    fn hash_chunk_produces_64_char_hex() {
        let chunk = crate::models::Chunk::new(0, b"test data".to_vec());
        let hashed = hash_chunk(chunk).unwrap();
        assert_eq!(hashed.hash.len(), 64);
        assert!(hashed.hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // hash_chunk_bytes is consistent with hash_chunk.
    #[test]
    fn hash_chunk_bytes_consistent_with_hash_chunk() {
        let bytes = b"consistency check".to_vec();
        let chunk = crate::models::Chunk::new(0, bytes.clone());
        let via_chunk = hash_chunk(chunk).unwrap().hash;
        let via_bytes = hash_chunk_bytes(&bytes).unwrap();
        assert_eq!(via_chunk, via_bytes);
    }

    // Known SHA-256 value for b"" (empty bytes).
    #[test]
    fn known_sha256_of_empty_bytes() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let got = hash_chunk_bytes(b"").unwrap();
        assert_eq!(got, expected);
    }

    // The chunk bytes are preserved after hashing (no mutation).
    #[test]
    fn hash_chunk_preserves_bytes() {
        let original = b"preserve me".to_vec();
        let chunk = crate::models::Chunk::new(0, original.clone());
        let hashed = hash_chunk(chunk).unwrap();
        assert_eq!(hashed.bytes, original);
    }
}
