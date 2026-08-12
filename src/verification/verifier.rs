use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::hash::Hash;

use crate::constants::{HEADER_SIZE, MAGIC, OBJECTS_DIR};
use crate::error::SnaprResult;
use crate::filesystem::hash::hash_chunk_bytes;
use crate::models::{ChunkVerifyResult, CompressionType, VerifyReport};
use crate::models::{FileEntry, FileVerifyReport, Snapshot, SnapshotVerifyReport, VerifyIssue};
use crate::storage::snapshots;

pub fn verify_chunk(hash: &str) -> SnaprResult<ChunkVerifyResult> {
    let object_path = format!("{}/{}", OBJECTS_DIR, hash);
    let object = fs::read(object_path).map_err(|_| VerifyIssue::MissingChunk {
        hash: hash.to_string(),
    })?;

    // 1. Basic object size validation
    if object.len() < HEADER_SIZE {
        return Err(VerifyIssue::InvalidHeader {
            hash: hash.to_string(),
        }
        .into());
    }

    // 2. Magic
    if &object[..MAGIC.len()] != MAGIC {
        return Err(VerifyIssue::InvalidHeader {
            hash: hash.to_string(),
        }
        .into());
    }

    // 3. Header
    let _version = object[5];
    let _flags = object[6];
    let compression = object[7];
    let original_size = u64::from_le_bytes(object[8..16].try_into()?);

    let compressed = &object[HEADER_SIZE..];
    // 4. Decompress according to compression type
    let contents = match compression {
        x if x == CompressionType::Zstd as u8 => zstd::decode_all(compressed)?,

        _ => {
            return Err(VerifyIssue::UnsupportedCompression {
                hash: hash.to_string(),
                compression,
            }
            .into());
        }
    };
    // 5. Verify original size
    if contents.len() as u64 != original_size {
        return Err(VerifyIssue::CorruptedChunk {
            hash: hash.to_string(),
        }
        .into());
    }

    // 6. Verify content hash
    let actual_hash = hash_chunk_bytes(&contents)?;

    if actual_hash != hash {
        return Err(VerifyIssue::HashMismatch {
            expected: hash.to_string(),
            actual: actual_hash,
        }
        .into());
    }

    Ok(ChunkVerifyResult {
        hash: hash.to_string(),
        original_size,
    })
}

pub fn verify_file(hashes: &[&String]) -> SnaprResult<FileVerifyReport> {
    let mut report = FileVerifyReport::default();
    let results = hashes
        .par_iter()
        .map(|hash| verify_chunk(hash))
        .collect::<SnaprResult<Vec<ChunkVerifyResult>>>()?;
    for result in results {
        report.record(&result);
    }

    Ok(report)
}

pub fn verify_snapshot(
    snapshot: &Snapshot,
    verified: &HashSet<&String>,
) -> SnaprResult<SnapshotVerifyReport> {
    let mut report = SnapshotVerifyReport::default();
    let results = snapshot
        .files
        .par_iter()
        .map(|FileEntry { chunk_hashes, .. }| {
            let unverified_hashes = chunk_hashes
                .iter()
                .filter(|hash| !verified.contains(*hash))
                // .cloned()
                .collect::<Vec<_>>();
            verify_file(&unverified_hashes)
        })
        .collect::<SnaprResult<Vec<FileVerifyReport>>>()?;
    for result in results {
        report.merge(&result);
    }
    Ok(report)
}

pub fn verify_repository(snapshots: &[Snapshot]) -> SnaprResult<VerifyReport> {
    let mut report = VerifyReport::default();
    let mut hash_set: HashSet<&String> =
        HashSet::with_capacity(snapshots.iter().map(|f| f.files.len()).sum());
    for snapshot in snapshots.iter() {
        let snapshot_report = verify_snapshot(snapshot, &hash_set)?;
        report.snapshots_checked += 1;
        report.files_checked += snapshot_report.files_checked;
        report.chunks_checked += snapshot_report.chunks_checked;
        report.bytes_verified += snapshot_report.bytes_verified;

        for file in &snapshot.files {
            for hash in &file.chunk_hashes {
                hash_set.insert(hash);
            }
        }
    }
    Ok(report)
}
