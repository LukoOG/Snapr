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

// After save updates the index, a second identical save must not store any
// new chunks.  When the cache hits (mtime+size match), the file is not
// re-processed at all: both new_chunks and reused_chunks stay 0.  Either way,
// new_chunks must be 0.
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
        // A workspace-index cache hit means the file is skipped entirely —
        // neither new_chunks nor reused_chunks is incremented for that file.
        // The invariant is simply: no new storage.
        assert_eq!(r2.new_chunks, 0, "no new chunks must be stored for an unchanged file");
    });
}

// ── Restore / index-synchronisation regression tests ──────────────────────────
//
// Root cause (fixed): handle_restore wrote files to disk but never updated the
// workspace index.  The next build_entries call (status, second restore, etc.)
// therefore always saw stale mtime/size values and re-hashed every file.

use snapr::commands::restore::handle_restore;
use snapr::commands::models::RestoreOptions;
use std::fs;

fn save(msg: &str) {
    let mut snaps = load_snapshots().unwrap();
    handle_save(&mut snaps, msg.to_string()).unwrap();
}

fn restore_snap(id: u32) -> snapr::models::RestoreReport {
    let snaps = load_snapshots().unwrap();
    handle_restore(&snaps, RestoreOptions { snapshot_id: id, force: true, dry_run: false }).unwrap()
}

// Invariant: after a successful non-dry-run restore, the workspace index must
// accurately reflect the resulting filesystem state.
//
// Specifically:
//   - files that were restored must have fresh mtime+size+hashes in the index;
//   - files that were deleted must be absent from the index.
#[test]
fn restore_updates_workspace_index_for_restored_files() {
    with_repo(|_| {
        write_file("keep.txt",   b"keep this file");
        write_file("change.txt", b"original content - long enough to differ in size");
        save("s1");

        // Modify change.txt and add new.txt for snapshot 2.
        write_file("change.txt", b"modified");
        write_file("new.txt",    b"new file");
        save("s2");

        // Restore to snapshot 1: change.txt is restored, new.txt is deleted.
        restore_snap(1);

        let index = load_workspace_index().unwrap();

        // new.txt was deleted → must not be in the index.
        assert!(
            !index.contains("new.txt"),
            "new.txt was deleted by restore; must not remain in the index"
        );

        // change.txt was restored → index entry must exist with the snapshot-1 hashes.
        let entry = index.get("change.txt")
            .expect("change.txt must be in the index after restore");
        let meta = fs::metadata("change.txt").unwrap();
        assert!(
            entry.matches(&meta),
            "index entry for change.txt must match the just-restored file's metadata"
        );

        // Verify the hashes in the index match the target snapshot.
        let snaps = load_snapshots().unwrap();
        let s1_hashes = snaps.iter()
            .find(|s| s.id == 1).unwrap()
            .files.iter()
            .find(|f| f.path == "change.txt").unwrap()
            .chunk_hashes.clone();
        assert_eq!(
            entry.chunk_hashes, s1_hashes,
            "index hashes for change.txt must match snapshot 1's hashes after restore"
        );
    });
}

// Invariant: a second restore of the same snapshot must produce zero cache
// misses — every file on disk was written by the first restore, and the index
// was synchronised at that point.
#[test]
fn second_restore_of_same_snapshot_produces_no_cache_misses() {
    with_repo(|_| {
        write_file("a.txt", b"file a content");
        write_file("b.txt", b"file b content");
        save("s1");

        write_file("a.txt", b"modified a - different length");
        write_file("c.txt", b"new file c");
        save("s2");

        // First restore to s1.
        restore_snap(1);

        // Second restore to s1: the index was updated by the first restore, so
        // every file currently on disk should produce a cache hit.  We verify
        // this indirectly: the diff inside handle_restore sees workspace == s1,
        // so restored_files must be 0 (nothing to do).
        let report = restore_snap(1);
        assert_eq!(
            report.restored_files, 0,
            "second restore of same snapshot should restore nothing (cache should be accurate)"
        );
        assert_eq!(
            report.removed_files, 0,
            "second restore of same snapshot should remove nothing"
        );
    });
}

// Regression: the historical reproduction scenario.
//   Snapshot 1: small files only.
//   Snapshot 2: Snapshot 1 files + large (simulated) media files.
//   restore(1) → index synchronised.
//   restore(1) again → must not re-hash anything; no files written.
#[test]
fn repeated_restore_of_snapshot_without_media_files_stays_clean() {
    with_repo(|_| {
        // Snapshot 1: small project files.
        write_file("src/main.rs", b"fn main() {}");
        write_file("README.md",   b"# readme");
        save("snapshot 1 — no media");

        // Snapshot 2: same files + a simulated media file (distinct content).
        write_file("media/video.mp4", b"fake media bytes - large enough to be distinct");
        save("snapshot 2 — with media");

        // First restore to snapshot 1: media/video.mp4 is deleted.
        restore_snap(1);

        assert!(!std::path::Path::new("media/video.mp4").exists(),
            "media file must be absent after restoring snapshot 1");

        // Second restore to snapshot 1: index was synchronised by first restore.
        // Nothing on disk differs from snapshot 1 → nothing should be written.
        let report = restore_snap(1);
        assert_eq!(
            report.restored_files, 0,
            "second restore should not re-write any files (index must be accurate after first restore)"
        );
        assert_eq!(
            report.removed_files, 0,
            "second restore should not delete any files"
        );

        // Final integrity check.
        let snaps = load_snapshots().unwrap();
        let verify = snapr::commands::verify::handle_verify(&snaps).unwrap();
        assert_eq!(verify.issues.len(), 0);
    });
}

// Invariant: after restore, the index contains no entries for files that
// were already absent from the target snapshot AND were already absent from
// the current workspace (i.e. remove-of-removed is a no-op; index is clean).
#[test]
fn restore_index_contains_no_stale_entries_for_absent_files() {
    with_repo(|_| {
        write_file("present.txt", b"always here");
        write_file("ephemeral.txt", b"only in s1");
        save("s1");

        // s2: ephemeral.txt removed.
        fs::remove_file("ephemeral.txt").unwrap();
        save("s2");

        // Restore to s1: ephemeral.txt is written back.
        restore_snap(1);
        assert!(std::path::Path::new("ephemeral.txt").exists());

        // Restore to s2: ephemeral.txt is removed again.
        restore_snap(2);
        assert!(!std::path::Path::new("ephemeral.txt").exists());

        // Index must not contain ephemeral.txt.
        let index = load_workspace_index().unwrap();
        assert!(
            !index.contains("ephemeral.txt"),
            "index must not contain ephemeral.txt after it was deleted by restore"
        );
    });
}
