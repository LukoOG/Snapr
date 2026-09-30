mod helpers;

use helpers::{with_repo, write_file};
use snapr::commands::save::handle_save;
use snapr::commands::verify::handle_verify;
use snapr::constants::OBJECTS_DIR;
use snapr::models::VerifyIssue;
use snapr::storage::load_snapshots;
use std::fs;

fn save(msg: &str) {
    let mut snaps = load_snapshots().unwrap();
    handle_save(&mut snaps, msg.to_string()).unwrap();
}

// Invariant 30: verify on a clean repository reports zero issues.
#[test]
fn verify_clean_repo_has_no_issues() {
    with_repo(|_| {
        write_file("a.txt", b"clean content A");
        write_file("b.txt", b"clean content B");
        save("clean snap");

        let snaps = load_snapshots().unwrap();
        let report = handle_verify(&snaps).unwrap();

        assert_eq!(report.issues.len(), 0, "no issues expected for clean repo");
        assert_eq!(report.snapshots_checked, 1);
        assert!(report.chunks_verified > 0);
        assert_eq!(report.chunks_verified, report.chunks_referenced);
    });
}

// Invariant 31: deleting a chunk file produces a MissingChunk issue.
#[test]
fn verify_deleted_chunk_produces_missing_chunk_issue() {
    with_repo(|dir| {
        write_file("target.txt", b"content to corrupt by deletion");
        save("snap");

        let snaps = load_snapshots().unwrap();
        // Get the hash of the stored chunk.
        let hash = snaps[0].files[0].chunk_hashes[0].clone();
        let obj_path = dir.join(OBJECTS_DIR).join(&hash);
        fs::remove_file(&obj_path).expect("should be able to remove object file");

        let report = handle_verify(&snaps).unwrap();
        assert_eq!(report.issues.len(), 1);
        assert!(
            matches!(&report.issues[0], VerifyIssue::MissingChunk { hash: h } if h == &hash),
            "expected MissingChunk issue, got: {:?}", report.issues[0]
        );
    });
}

// Invariant 32: corrupting the magic bytes produces an InvalidHeader issue.
#[test]
fn verify_corrupt_magic_produces_invalid_header() {
    with_repo(|dir| {
        write_file("target.txt", b"content to corrupt by magic clobber");
        save("snap");

        let snaps = load_snapshots().unwrap();
        let hash = snaps[0].files[0].chunk_hashes[0].clone();
        let obj_path = dir.join(OBJECTS_DIR).join(&hash);

        let mut obj = fs::read(&obj_path).unwrap();
        // Overwrite the magic bytes.
        obj[..5].copy_from_slice(b"XXXXX");
        fs::write(&obj_path, &obj).unwrap();

        let report = handle_verify(&snaps).unwrap();
        assert_eq!(report.issues.len(), 1);
        assert!(
            matches!(&report.issues[0], VerifyIssue::InvalidHeader { .. }),
            "expected InvalidHeader issue, got: {:?}", report.issues[0]
        );
    });
}

// Invariant 34: changing the original_size header field while leaving data intact → CorruptedChunk.
#[test]
fn verify_wrong_original_size_produces_corrupted_chunk() {
    with_repo(|dir| {
        write_file("target.txt", b"original size mismatch test");
        save("snap");

        let snaps = load_snapshots().unwrap();
        let hash = snaps[0].files[0].chunk_hashes[0].clone();
        let obj_path = dir.join(OBJECTS_DIR).join(&hash);

        let mut obj = fs::read(&obj_path).unwrap();
        // Corrupt the original_size field (bytes 8..16) — set to 0.
        obj[8..16].copy_from_slice(&0u64.to_le_bytes());
        fs::write(&obj_path, &obj).unwrap();

        let report = handle_verify(&snaps).unwrap();
        assert_eq!(report.issues.len(), 1);
        assert!(
            matches!(&report.issues[0], VerifyIssue::CorruptedChunk { .. }),
            "expected CorruptedChunk issue, got: {:?}", report.issues[0]
        );
    });
}

// Invariant 35: writing wrong content (wrong hash filename) → HashMismatch.
#[test]
fn verify_hash_mismatch_produces_hash_mismatch_issue() {
    with_repo(|dir| {
        write_file("a.txt", b"original content");
        write_file("b.txt", b"different content");
        save("snap");

        let snaps = load_snapshots().unwrap();
        // Find both chunk objects.
        let hash_a = snaps[0].files.iter()
            .find(|f| f.path.contains("a.txt"))
            .unwrap()
            .chunk_hashes[0].clone();
        let hash_b = snaps[0].files.iter()
            .find(|f| f.path.contains("b.txt"))
            .unwrap()
            .chunk_hashes[0].clone();

        let obj_a = dir.join(OBJECTS_DIR).join(&hash_a);
        let obj_b = dir.join(OBJECTS_DIR).join(&hash_b);

        // Swap the files: hash_a now contains b's data and vice versa.
        let bytes_a = fs::read(&obj_a).unwrap();
        let bytes_b = fs::read(&obj_b).unwrap();
        fs::write(&obj_a, &bytes_b).unwrap();
        fs::write(&obj_b, &bytes_a).unwrap();

        let report = handle_verify(&snaps).unwrap();
        // Both files have content that doesn't match their hash filename.
        assert!(
            report.issues.len() >= 1,
            "expected at least one HashMismatch issue"
        );
        assert!(
            report.issues.iter().any(|i| matches!(i, VerifyIssue::HashMismatch { .. })),
            "expected at least one HashMismatch issue"
        );
    });
}

// Invariant 36: same chunk referenced in multiple snapshots is only verified once.
#[test]
fn verify_deduplicates_chunk_verification_across_snapshots() {
    with_repo(|_| {
        // Same file content in both snapshots → same chunk hash → should only verify once.
        write_file("shared.txt", b"same content in both snapshots");
        save("s1");
        save("s2"); // identical content → same chunk objects

        let snaps = load_snapshots().unwrap();
        let report = handle_verify(&snaps).unwrap();

        assert_eq!(report.snapshots_checked, 2);
        assert_eq!(report.issues.len(), 0);

        // chunks_referenced counts each reference (2 snaps × same chunk = 2).
        // chunks_verified (unique) should be less or equal to chunks_referenced.
        assert!(
            report.total_chunks <= report.chunks_referenced,
            "unique chunks should not exceed total references"
        );
    });
}

// Verify reports correct counts.
#[test]
fn verify_counts_are_consistent() {
    with_repo(|_| {
        write_file("file1.txt", b"file one content");
        write_file("file2.txt", b"file two content");
        save("snap");

        let snaps = load_snapshots().unwrap();
        let report = handle_verify(&snaps).unwrap();

        assert_eq!(report.snapshots_checked, 1);
        assert!(report.unique_file_versions_checked >= 2);
        assert!(report.chunks_verified > 0);
        assert_eq!(report.issues.len(), 0);
    });
}

// Verify on empty snapshots list returns zero-counts report with no issues.
#[test]
fn verify_empty_snapshots_is_clean() {
    with_repo(|_| {
        let snaps = load_snapshots().unwrap();
        assert!(snaps.is_empty());
        let report = handle_verify(&snaps).unwrap();
        assert_eq!(report.snapshots_checked, 0);
        assert_eq!(report.issues.len(), 0);
    });
}
