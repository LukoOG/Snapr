/// End-to-end integration test: exercises the full workflow.
/// init → save → modify → save → diff → restore → verify

mod helpers;

use helpers::{with_repo, write_file, read_file_bytes};
use snapr::commands::diff::handle_diff;
use snapr::commands::restore::handle_restore;
use snapr::commands::models::RestoreOptions;
use snapr::commands::save::handle_save;
use snapr::commands::status::handle_status;
use snapr::commands::verify::handle_verify;
use snapr::processing::build_entries;
use snapr::storage::{load_config, load_snapshots};
use std::fs;

#[test]
fn end_to_end_init_save_modify_save_diff_restore_verify() {
    with_repo(|_| {
        // ── Step 1: Create initial workspace ──
        // Note: use content of different sizes so the workspace-index cache
        // cannot produce a false hit when writes happen within the same second.
        write_file("project/main.rs", b"fn main() { println!(\"version one\"); }");
        write_file("project/lib.rs", b"pub fn helper() {}");
        write_file("README.md", b"# My Project v1");

        // ── Step 2: First save ──
        let mut snaps = load_snapshots().unwrap();
        let r1 = handle_save(&mut snaps, "initial commit".to_string()).unwrap();
        assert_eq!(r1.files_processed, 3);
        assert!(r1.new_chunks > 0);

        let config = load_config().unwrap();
        assert_eq!(config.current_snapshot, Some(1));

        // ── Step 3: Modify workspace ──
        write_file("project/main.rs", b"fn main() { println!(\"version two - updated\"); }");
        fs::remove_file("project/lib.rs").unwrap();
        write_file("project/new_module.rs", b"pub mod new_mod {}");

        // ── Step 4: Second save ──
        let mut snaps = load_snapshots().unwrap();
        let r2 = handle_save(&mut snaps, "second commit".to_string()).unwrap();
        // main.rs changed → new chunk; new_module.rs is new → new chunk.
        // README.md is unchanged and gets a workspace-index cache hit, so it
        // contributes 0 to both new_chunks and reused_chunks (not re-processed).
        assert!(r2.new_chunks > 0, "new chunks expected for changed/added files");
        // files_processed is at least the non-cached files (main.rs + new_module.rs).
        assert!(r2.files_processed >= 2);

        let config = load_config().unwrap();
        assert_eq!(config.current_snapshot, Some(2));

        // ── Step 5: Diff between snapshot 1 and 2 ──
        let snaps = load_snapshots().unwrap();
        let diff = handle_diff(&snaps, 1, 2).unwrap();

        // main.rs changed → modified.
        assert!(
            diff.modified.iter().any(|p| p.contains("main.rs")),
            "main.rs should be modified; diff.modified={:?}", diff.modified
        );
        // lib.rs removed in s2.
        assert!(
            diff.removed.iter().any(|p| p.contains("lib.rs")),
            "lib.rs should be removed; diff.removed={:?}", diff.removed
        );
        // new_module.rs added in s2.
        assert!(
            diff.added.iter().any(|p| p.contains("new_module.rs")),
            "new_module.rs should be added; diff.added={:?}", diff.added
        );

        // ── Step 6: Status (must show workspace == snapshot 2 after save) ──
        let entries = build_entries().unwrap();
        let snaps = load_snapshots().unwrap();
        let status = handle_status(&snaps, entries).unwrap();
        // Workspace should equal snapshot 2 (we haven't changed anything since s2).
        assert!(
            status.added.is_empty() && status.modified.is_empty() && status.removed.is_empty(),
            "workspace should be clean relative to current snapshot after save; status={status:?}"
        );

        // ── Step 7: Restore to snapshot 1 ──
        let snaps = load_snapshots().unwrap();
        let report = handle_restore(
            &snaps,
            RestoreOptions { snapshot_id: 1, force: false, dry_run: false },
        ).unwrap();
        assert!(!report.dry_run);

        // Check filesystem reflects snapshot 1.
        assert_eq!(
            read_file_bytes("project/main.rs"),
            b"fn main() { println!(\"version one\"); }",
            "main.rs should be restored to version one"
        );
        assert_eq!(
            read_file_bytes("project/lib.rs"),
            b"pub fn helper() {}",
            "lib.rs should be restored"
        );
        assert!(
            !std::path::Path::new("project/new_module.rs").exists(),
            "new_module.rs should not exist in snapshot 1"
        );

        let config = load_config().unwrap();
        assert_eq!(config.current_snapshot, Some(1));

        // ── Step 8: Verify repository integrity ──
        let snaps = load_snapshots().unwrap();
        let verify = handle_verify(&snaps).unwrap();
        assert_eq!(
            verify.issues.len(),
            0,
            "no integrity issues expected: {:?}", verify.issues
        );
        assert_eq!(verify.snapshots_checked, 2);
        assert!(verify.chunks_verified > 0);
    });
}

// Edge case: diff with reversed IDs (old > new) is valid — diff is directional.
// Content versions are different sizes so the workspace-index cache cannot
// produce a false hit when both writes happen within the same second.
#[test]
fn diff_is_directional() {
    with_repo(|_| {
        write_file("f.txt", b"version one content");
        let mut snaps = load_snapshots().unwrap();
        handle_save(&mut snaps, "s1".to_string()).unwrap();

        write_file("f.txt", b"version two - different length content");
        let mut snaps = load_snapshots().unwrap();
        handle_save(&mut snaps, "s2".to_string()).unwrap();

        let snaps = load_snapshots().unwrap();

        // 1 → 2: f.txt is modified.
        let forward = handle_diff(&snaps, 1, 2).unwrap();
        assert_eq!(forward.modified, vec!["f.txt"]);

        // 2 → 1: f.txt is also modified (content changed both ways).
        let backward = handle_diff(&snaps, 2, 1).unwrap();
        assert_eq!(backward.modified, vec!["f.txt"]);
    });
}

// diff with id=0 returns an error.
#[test]
fn diff_with_zero_id_errors() {
    with_repo(|_| {
        write_file("f.txt", b"data");
        let mut snaps = load_snapshots().unwrap();
        handle_save(&mut snaps, "s1".to_string()).unwrap();

        let snaps = load_snapshots().unwrap();
        assert!(handle_diff(&snaps, 0, 1).is_err());
        assert!(handle_diff(&snaps, 1, 0).is_err());
    });
}

// history errors when no snapshots exist.
#[test]
fn history_errors_when_no_snapshots() {
    with_repo(|_| {
        let snaps = load_snapshots().unwrap();
        let result = snapr::commands::history::handle_history(&snaps);
        assert!(result.is_err());
    });
}
