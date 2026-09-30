mod helpers;

use helpers::with_repo;
use snapr::constants::OBJECTS_DIR;
use snapr::filesystem::compress::compress_chunk;
use snapr::filesystem::hash::hash_chunk;
use snapr::models::Chunk;
use snapr::storage::chunk::{store_chunk, read_chunk};
use std::fs;

fn make_compressed(data: &[u8]) -> snapr::models::CompressedChunk {
    let chunk = Chunk::new(0, data.to_vec());
    let hashed = hash_chunk(chunk).unwrap();
    compress_chunk(hashed).unwrap()
}

// Invariant 1 & 9: full round-trip via store + read gives back original bytes.
#[test]
fn store_then_read_recovers_original_bytes() {
    with_repo(|_| {
        let original = b"round-trip test for store+read";
        let compressed = make_compressed(original);
        let hash = compressed.hash.clone();
        store_chunk(compressed).unwrap();

        let recovered = read_chunk(&hash).unwrap();
        assert_eq!(recovered, original);
    });
}

// Invariant 3: store_chunk is idempotent — second write returns stored=false.
#[test]
fn store_chunk_is_idempotent() {
    with_repo(|_| {
        let compressed = make_compressed(b"idempotent content");
        let hash = compressed.hash.clone();

        // Clone object_bytes for second call (compressed is consumed by first).
        let object_bytes2 = compressed.object_bytes.clone();
        let original_size2 = compressed.original_size;
        let index2 = compressed.index;

        let r1 = store_chunk(compressed).unwrap();
        assert!(r1.stored, "first store should write the object");

        let r2 = store_chunk(snapr::models::CompressedChunk {
            index: index2,
            hash: hash.clone(),
            object_bytes: object_bytes2,
            original_size: original_size2,
        }).unwrap();
        assert!(!r2.stored, "second store of same chunk should not write again");
    });
}

// Invariant 4a: read_chunk on a missing hash returns a clear error.
#[test]
fn read_chunk_missing_hash_errors() {
    with_repo(|_| {
        let result = read_chunk("a".repeat(64).as_str());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.to_lowercase().contains("missing"),
            "error should mention 'missing': {msg}"
        );
    });
}

// Invariant 4b: read_chunk on a truncated file (shorter than header) errors.
#[test]
fn read_chunk_truncated_file_errors() {
    with_repo(|dir| {
        // Write a fake too-short object file.
        let fake_hash = "b".repeat(64);
        let obj_path = dir.join(OBJECTS_DIR).join(&fake_hash);
        fs::write(&obj_path, b"SHORT").unwrap(); // only 5 bytes

        let result = read_chunk(&fake_hash);
        assert!(result.is_err(), "truncated object should fail to read");
    });
}

// Invariant 4c: read_chunk on a file with bad magic bytes errors.
#[test]
fn read_chunk_bad_magic_errors() {
    with_repo(|dir| {
        let fake_hash = "c".repeat(64);
        let obj_path = dir.join(OBJECTS_DIR).join(&fake_hash);
        // 16-byte header but wrong magic.
        let mut bad_obj = vec![0u8; 16];
        bad_obj[..5].copy_from_slice(b"XBADX");
        bad_obj[7] = 1; // Zstd compression
        fs::write(&obj_path, bad_obj).unwrap();

        let result = read_chunk(&fake_hash);
        assert!(result.is_err(), "bad-magic object should fail to read");
    });
}

// ChunkStoreResult carries the original_size correctly.
#[test]
fn store_chunk_result_carries_original_size() {
    with_repo(|_| {
        let data = b"size tracking test";
        let compressed = make_compressed(data);
        let expected_original = data.len();
        let result = store_chunk(compressed).unwrap();
        assert_eq!(result.original_size, expected_original);
    });
}

// ChunkStoreResult hash matches the expected SHA-256 of the content.
#[test]
fn store_chunk_result_hash_is_sha256_of_original() {
    with_repo(|_| {
        let data = b"hash check for store result";
        let chunk = Chunk::new(0, data.to_vec());
        let hashed = hash_chunk(chunk).unwrap();
        let expected_hash = hashed.hash.clone();
        let compressed = compress_chunk(hashed).unwrap();
        let result = store_chunk(compressed).unwrap();
        assert_eq!(result.hash, expected_hash);
    });
}
