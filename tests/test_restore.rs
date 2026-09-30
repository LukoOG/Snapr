mod helpers;

use helpers::{with_repo, write_file, read_file_bytes};
use snapr::commands::save::handle_save;
use snapr::commands::restore::handle_restore;
use snapr::commands::models::RestoreOptions;
use snapr::storage::{load_snapshots, load_config};
use std::fs;

fn save(msg: &str) {
    let mut snaps = load_snapshots().unwrap();
    handle_save(&mut snaps, msg.to_string()).unwrap();
}

fn restore(id: u32, force: bool, dry_run: bool) -> snapr::models::RestoreReport {
    let snaps = load_snapshots().unwrap();
    handle_restore(&snaps, RestoreOptions { snapshot_id: id, force, dry_run }).unwrap()
}

// Invariant 24: restore writes every file in the target snapshot to disk.
// Invariant 29: after restore, config.current_snapshot == target snapshot id.
#[test]
fn restore_writes_target_files_to_disk() {
    with_repo(|_| {
        write_file("file_a.txt", b"content of A");
        write_file("file_b.txt", b"content of B");
        save("snapshot 1");

        // Now modify the files.
        write_file("file_a.txt", b"MODIFIED A");
        fs::remove_file("file_b.txt").unwrap();
        write_file("file_c.txt", b"new file C");
        save("snapshot 2");

        // Restore to snapshot 1.
        restore(1, true, false);

        assert_eq!(read_file_bytes("file_a.txt"), b"content of A");
        assert_eq!(read_file_bytes("file_b.txt"), b"content of B");
        assert!(
            !std::path::Path::new("file_c.txt").exists(),
            "file_c.txt should not exist after restoring to snapshot 1"
        );

        let config = load_config().unwrap();
        assert_eq!(config.current_snapshot, Some(1));
    });
}

// Invariant 25: files absent in target but present on disk are deleted.
#[test]
fn restore_removes_files_not_in_target() {
    with_repo(|_| {
        write_file("keep.txt", b"keep me");
        save("s1");

        // Add extra file and save snapshot 2.
        write_file("extra.txt", b"I should be removed");
        save("s2");

        // Restore back to s1 — extra.txt should be deleted.
        restore(1, true, false);

        assert!(std::path::Path::new("keep.txt").exists());
        assert!(
            !std::path::Path::new("extra.txt").exists(),
            "extra.txt should be removed when restoring to s1"
        );
    });
}

// Invariant 26: unchanged files count as skipped.
#[test]
fn restore_counts_skipped_unchanged_files() {
    with_repo(|_| {
        write_file("unchanged.txt", b"same");
        write_file("will_change.txt", b"original");
        save("s1");

        write_file("will_change.txt", b"changed");
        save("s2");

        // Restore to s1 from s2. unchanged.txt is identical between the two,
        // will_change.txt differs.
        let report = restore(1, true, false);

        // 1 modified file restored, 1 unchanged skipped.
        assert_eq!(report.restored_files, 1, "only the changed file should be restored");
        assert_eq!(report.skipped_files, 1, "the unchanged file should be skipped");
    });
}

// Invariant 27: --dry-run makes zero filesystem changes.
#[test]
fn restore_dry_run_makes_no_filesystem_changes() {
    with_repo(|_| {
        write_file("original.txt", b"v1");
        save("s1");

        write_file("original.txt", b"v2");
        save("s2");

        // Capture file contents before dry-run.
        let before = read_file_bytes("original.txt");

        let report = restore(1, true, true);
        assert!(report.dry_run);

        // File must not change.
        let after = read_file_bytes("original.txt");
        assert_eq!(before, after, "dry-run must not modify files on disk");

        // Config must not change.
        let config = load_config().unwrap();
        assert_eq!(
            config.current_snapshot,
            Some(2),
            "dry-run must not update current_snapshot in config"
        );
    });
}

// Invariant 28: --force bypasses "already on this snapshot" error.
#[test]
fn restore_force_bypasses_already_on_snapshot_error() {
    with_repo(|_| {
        write_file("f.txt", b"data");
        save("s1");

        // We are on snapshot 1. Without --force, restoring to 1 should error.
        let snaps = load_snapshots().unwrap();
        let result = handle_restore(
            &snaps,
            RestoreOptions { snapshot_id: 1, force: false, dry_run: false },
        );
        assert!(result.is_err(), "restoring to current snapshot without --force should fail");

        // With --force it should succeed.
        let snaps = load_snapshots().unwrap();
        let result = handle_restore(
            &snaps,
            RestoreOptions { snapshot_id: 1, force: true, dry_run: false },
        );
        assert!(result.is_ok(), "restoring to current snapshot with --force should succeed");
    });
}

// Restore of a non-existent snapshot ID returns an error.
#[test]
fn restore_unknown_snapshot_id_errors() {
    with_repo(|_| {
        write_file("f.txt", b"data");
        save("s1");

        let snaps = load_snapshots().unwrap();
        let result = handle_restore(
            &snaps,
            RestoreOptions { snapshot_id: 999, force: false, dry_run: false },
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.to_lowercase().contains("not found"),
            "expected 'not found': {msg}"
        );
    });
}

// Invariant: restored_bytes matches the actual bytes written to disk.
#[test]
fn restore_restored_bytes_matches_disk() {
    with_repo(|_| {
        let content = b"precisely measured content";
        write_file("measured.txt", content);
        save("s1");

        // Modify and create snapshot 2 so we have something to restore from.
        write_file("measured.txt", b"changed");
        save("s2");

        let report = restore(1, true, false);
        assert_eq!(
            report.restored_bytes,
            content.len() as u64,
            "restored_bytes should equal the actual bytes written"
        );
    });
}

// Restore handles subdirectory paths correctly (directory creation).
#[test]
fn restore_creates_parent_directories() {
    with_repo(|_| {
        write_file("subdir/deep/nested.txt", b"nested content");
        save("s1");

        // Remove the file and its parent directories.
        fs::remove_dir_all("subdir").unwrap();
        assert!(!std::path::Path::new("subdir").exists());

        restore(1, true, false);

        assert_eq!(read_file_bytes("subdir/deep/nested.txt"), b"nested content");
    });
}
