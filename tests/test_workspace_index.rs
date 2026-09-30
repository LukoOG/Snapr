/// Workspace-index regression tests.
///
/// The critical invariant: `build_entries()` (used by `status` and `restore` for
/// workspace hashing) MUST NOT write to the workspace index. Only
/// `build_snapshot_entries()` (used by `save`) is allowed to update the index.
///
/// If `build_entries` polluted the index, a subsequent `save` would see a
/// cache hit and skip re-processing the file — meaning its chunks would never
/// be stored, but the snapshot would still reference them.

mod helpers;

use helpers::{with_repo, write_file};
use snapr::commands::save::handle_save;
use snapr::commands::status::handle_status;
use snapr::processing::build_entries;
use snapr::storage::load_snapshots;
use snapr::storage::workspace_index::load_workspace_index;

// Regression: build_entries (hash-only mode) must not modify the workspace index.
#[test]
fn build_entries_does_not_pollute_workspace_index() {
    with_repo(|_| {
        write_file("tracked.txt", b"track this file");

        // Read the index before calling build_entries (should be empty).
        let index_before = load_workspace_index().unwrap();
        let len_before = index_before.len();

        // Call build_entries (as status/restore does).
        let _ = build_entries().unwrap();

        // Read the index after — must be unchanged.
        let index_after = load_workspace_index().unwrap();
        assert_eq!(
            index_after.len(),
            len_before,
            "build_entries must not add entries to the workspace index"
        );
        assert!(
            !index_after.contains("tracked.txt"),
            "build_entries must not insert 'tracked.txt' into the workspace index"
        );
    });
}

// Regression: calling build_entries followed by build_snapshot_entries must
// still process and store the file (no false cache hit from a hash-only pass).
#[test]
fn build_entries_followed_by_save_stores_all_chunks() {
    with_repo(|_| {
        write_file("important.txt", b"must be stored, not skipped");

        // Simulate what `status` does: hash-only pass.
        let _ = build_entries().unwrap();

        // Now save — all chunks must actually be stored.
        let mut snaps = load_snapshots().unwrap();
        let report = handle_save(&mut snaps, "after status".to_string()).unwrap();

        assert!(
            report.new_chunks > 0,
            "save after build_entries should store new chunks, not skip them"
        );
        assert_eq!(
            report.reused_chunks, 0,
            "no chunks should be reported as reused on a fresh save"
        );

        // Verify the stored objects are valid (the chunks are actually on disk).
        let snaps = load_snapshots().unwrap();
        let verify_report = snapr::commands::verify::handle_verify(&snaps).unwrap();
        assert_eq!(
            verify_report.issues.len(),
            0,
            "repository should be clean: {:?}",
            verify_report.issues
        );
    });
}

// Regression: status command compares against the current snapshot without
// modifying the workspace index.
#[test]
fn status_does_not_modify_workspace_index() {
    with_repo(|_| {
        write_file("status_test.txt", b"initial content");
        let mut snaps = load_snapshots().unwrap();
        handle_save(&mut snaps, "initial".to_string()).unwrap();

        // Modify the file.
        write_file("status_test.txt", b"modified content");

        // Read the index state after save (has one entry).
        let index_before_status = load_workspace_index().unwrap();
        let entry_before = index_before_status.get("status_test.txt").unwrap().clone();

        // Run status (via build_entries, same as handle_status does).
        let entries = build_entries().unwrap();
        let snaps = load_snapshots().unwrap();
        let _ = handle_status(&snaps, entries).unwrap();

        // Index must be exactly the same after status.
        let index_after_status = load_workspace_index().unwrap();
        let entry_after = index_after_status.get("status_test.txt").unwrap();

        assert_eq!(
            entry_before.chunk_hashes,
            entry_after.chunk_hashes,
            "status must not update the workspace index entry's chunk hashes"
        );
        assert_eq!(
            entry_before.modified,
            entry_after.modified,
            "status must not update the workspace index entry's mtime"
        );
    });
}

// After save updates the index, a second identical save sees a cache hit
// and reuses all chunks.
#[test]
fn save_cache_hit_reuses_chunks_on_second_save() {
    with_repo(|_| {
        write_file("cached.txt", b"content to cache");

        let mut snaps = load_snapshots().unwrap();
        let r1 = handle_save(&mut snaps, "first".to_string()).unwrap();
        assert!(r1.new_chunks > 0);

        // Second save with no file changes.
        let mut snaps = load_snapshots().unwrap();
        let r2 = handle_save(&mut snaps, "second".to_string()).unwrap();
        assert_eq!(r2.new_chunks, 0, "no new chunks expected on unchanged file");
        assert!(r2.reused_chunks > 0, "all chunks should be reused");
    });
}
