use crate::constants::{HEADER_SIZE, MAGIC, OBJECTS_DIR};
use crate::error::SnaprResult;
use crate::filesystem::hash::hash_chunk_bytes;
use crate::models::{ChunkVerificationResult, ChunkVerifyResult, CompressionType, VerifyReport};
use crate::models::{FileEntry, FileVerifyReport, Snapshot, SnapshotVerifyReport, VerifyIssue};
use crate::scoped_timer;
use crate::storage::models::ObjectHeader;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use super::models::verify::FileVerificationKey;

pub fn verify_chunk(hash: &str) -> SnaprResult<ChunkVerificationResult> {
    let object_path = format!("{}/{}", OBJECTS_DIR, hash);
    let object = fs::read(object_path).map_err(|_| VerifyIssue::MissingChunk {
        hash: hash.to_string(),
    })?;

    
    // 1. Basic object size validation
    if object.len() < HEADER_SIZE {
        return Ok(ChunkVerificationResult::Issue(VerifyIssue::InvalidHeader {
            hash: hash.to_string(),
        }));
    }
    
    // 2. Magic
    if &object[..MAGIC.len()] != MAGIC {
        return Ok(ChunkVerificationResult::Issue(VerifyIssue::InvalidHeader {
            hash: hash.to_string(),
        }));
    }
    //TODO: Get object header values from ObjectHeader::extract method
    // 3. Object
    let ObjectHeader {
        version: _,
        flags: _,
        compression,
        original_size,
    } = match ObjectHeader::extract(&object) {
        Some(v) => v,
        None => {
            return Ok(ChunkVerificationResult::Issue(VerifyIssue::InvalidHeader {
                hash: hash.to_string(),
            }));
        }
    };

    let compressed = &object[HEADER_SIZE..];
    // 4. Decompress according to compression type
    let contents = match compression {
        x if x == CompressionType::Zstd => zstd::decode_all(compressed)?,

        _ => {
            return Ok(ChunkVerificationResult::Issue(
                VerifyIssue::UnsupportedCompression {
                    hash: hash.to_string(),
                    compression: compression.into(),
                },
            ));
        }
    };
    // 5. Verify original size
    if contents.len() as u64 != original_size {
        return Ok(ChunkVerificationResult::Issue(
            VerifyIssue::CorruptedChunk {
                hash: hash.to_string(),
            },
        ));
    }

    // 6. Verify content hash
    let actual_hash = hash_chunk_bytes(&contents)?;

    if actual_hash != hash {
        return Ok(ChunkVerificationResult::Issue(VerifyIssue::HashMismatch {
            expected: hash.to_string(),
            actual: actual_hash,
        }));
    }

    Ok(ChunkVerificationResult::Verified(ChunkVerifyResult {
        hash: hash.to_string(),
        original_size,
    }))
}

pub fn verify_file(hashes: &[&String]) -> SnaprResult<FileVerifyReport> {
    let mut report = FileVerifyReport::default();
    let results = hashes
        .par_iter()
        .map(|hash| verify_chunk(hash))
        .collect::<SnaprResult<Vec<ChunkVerificationResult>>>()?;
    for result in results {
        report.record(result);
    }

    Ok(report)
}

pub fn verify_snapshot(
    snapshot: &Snapshot,
    verified: &HashSet<&String>,
) -> SnaprResult<SnapshotVerifyReport> {
    scoped_timer!("process snapshot {}", snapshot.id);
    let mut report = SnapshotVerifyReport::default();
    let results = snapshot
        .files
        .par_iter()
        .map(|FileEntry { chunk_hashes, .. }| {
            let unverified_hashes = chunk_hashes
                .iter()
                .filter(|hash| !verified.contains(*hash))
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
    let capacity = snapshots.iter().map(|f| f.files.len()).sum();
    let mut verified_chunks: HashSet<&String> =
        HashSet::with_capacity(capacity);
    let mut verified_files: HashSet<FileVerificationKey> =
        HashSet::with_capacity(capacity);
    for snapshot in snapshots.iter() {
        let snapshot_report = verify_snapshot(snapshot, &verified_chunks)?;
        // report.chunks_referenced += snapshot.files.iter().map(|f| f.chunk_hashes.len()).sum::<usize>();
        report.snapshots_checked += 1;
        report.merge(&snapshot_report);

        for file in &snapshot.files {
            report.chunks_referenced += file.chunk_hashes.len();
            verified_files.insert(FileVerificationKey {
                path: file.path.clone(),
                chunk_hashes: file.chunk_hashes.clone(),
            });
            for hash in &file.chunk_hashes {
                verified_chunks.insert(hash);
            }
        }
    }
    report.total_chunks = verified_chunks.len();
    report.files_checked = verified_files.len();
    Ok(report)
}
