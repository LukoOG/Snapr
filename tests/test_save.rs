mod helpers;

use helpers::{with_repo, write_file};
use snapr::commands::save::handle_save;
use snapr::storage::{load_snapshots, load_config};

// Invariant 14: after save, snapshots.json contains the new snapshot.
// Invariant 15: after save, config.current_snapshot matches the new snapshot ID.
#[test]
fn save_creates_snapshot_with_correct_id_and_config() {
    with_repo(|_| {
        write_file("hello.txt", b"hello world");
        let mut snapshots = load_snapshots().unwrap();
        let report = handle_save(&mut snapshots, "first snapshot".to_string()).unwrap();

        // Reload from disk to verify persistence.
        let persisted = load_snapshots().unwrap();
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].id, 1);
        assert_eq!(persisted[0].message, "first snapshot");

        // Config tracks current snapshot.
        let config = load_config().unwrap();
        assert_eq!(config.get_current_snapshot_id(), Some(1));

        // Report shows one file processed and non-zero new chunks.
        assert_eq!(report.files_processed, 1);
        assert!(report.new_chunks > 0);
        assert_eq!(report.reused_chunks, 0);
    });
}

// Invariant 18: IDs are always max(existing) + 1.
#[test]
fn save_assigns_sequential_ids() {
    with_repo(|_| {
        write_file("a.txt", b"aaa");
        let mut snaps = load_snapshots().unwrap();
        handle_save(&mut snaps, "s1".to_string()).unwrap();

        write_file("b.txt", b"bbb");
        let mut snaps = load_snapshots().unwrap();
        handle_save(&mut snaps, "s2".to_string()).unwrap();

        write_file("c.txt", b"ccc");
        let mut snaps = load_snapshots().unwrap();
        handle_save(&mut snaps, "s3".to_string()).unwrap();

        let snaps = load_snapshots().unwrap();
        let ids: Vec<u32> = snaps.iter().map(|s| s.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
        let config = load_config().unwrap();
        assert_eq!(config.get_current_snapshot_id(), Some(3));
    });
}

// Objects are stored in .snapr/objects/ after save.
#[test]
fn save_writes_objects_to_disk() {
    with_repo(|dir| {
        write_file("data.bin", b"unique content for object storage test");
        let mut snaps = load_snapshots().unwrap();
        handle_save(&mut snaps, "test".to_string()).unwrap();

        let objects_dir = dir.join(".snapr/objects");
        let entries: Vec<_> = std::fs::read_dir(&objects_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(!entries.is_empty(), "objects directory should not be empty after save");
    });
}

// Deduplication: saving the same file twice stores its chunk only once.
// Invariant 3 (store_chunk idempotency from the save perspective).
//
// Note on report semantics:
//   - A workspace-index cache HIT means the file is not re-processed at all.
//     Both new_chunks and reused_chunks stay 0 for that file in the report.
//   - A workspace-index cache MISS but object-store HIT means reused_chunks++.
//   Both paths result in zero new storage — this test validates the latter
//   property (no new chunks written), which holds regardless of which path
//   was taken.
#[test]
fn save_deduplicates_unchanged_file() {
    with_repo(|_| {
        write_file("dup.txt", b"content that will be saved twice");
        let mut snaps = load_snapshots().unwrap();
        let r1 = handle_save(&mut snaps, "first".to_string()).unwrap();
        assert!(r1.new_chunks > 0);

        // Second save with identical content.
        let mut snaps = load_snapshots().unwrap();
        let r2 = handle_save(&mut snaps, "second".to_string()).unwrap();
        // No new chunks should be written — content is identical.
        // Whether this is a cache hit (reused_chunks==0) or a store-level
        // dedup (reused_chunks>0) depends on mtime granularity; either way
        // new_chunks must be 0.
        assert_eq!(r2.new_chunks, 0, "no new chunks should be stored for unchanged content");
    });
}

// Snapshot stats are correctly populated.
#[test]
fn save_snapshot_stats_are_correct() {
    with_repo(|_| {
        let content = b"stats test content";
        write_file("stats.txt", content);
        let mut snaps = load_snapshots().unwrap();
        handle_save(&mut snaps, "stats".to_string()).unwrap();

        let snaps = load_snapshots().unwrap();
        let s = &snaps[0];
        assert_eq!(s.stats.file_count, 1);
        assert!(s.stats.chunk_count >= 1);
        assert_eq!(s.stats.workspace_bytes, content.len() as u64);
        assert!(s.stats.new_chunks >= 1);
        assert_eq!(s.stats.reused_chunks, 0);
    });
}

// After save, workspace_index is updated with the saved file's entry.
#[test]
fn save_updates_workspace_index() {
    with_repo(|_| {
        write_file("indexed.txt", b"index me");
        let mut snaps = load_snapshots().unwrap();
        handle_save(&mut snaps, "idx".to_string()).unwrap();

        let index = snapr::storage::workspace_index::load_workspace_index().unwrap();
        assert!(
            index.contains("indexed.txt"),
            "workspace index should contain 'indexed.txt'"
        );
    });
}

// Files are processed in parallel. When several new files share a chunk,
// exactly one worker may report it as new; the rest must report it as reused,
// and new_storage_bytes must equal what was actually written to disk.
#[test]
fn save_counts_chunk_shared_by_parallel_files_once() {
    with_repo(|dir| {
        const COPIES: usize = 16;
        for i in 0..COPIES {
            write_file(&format!("copy_{i}.txt"), b"identical content in every file");
        }

        let mut snaps = load_snapshots().unwrap();
        let report = handle_save(&mut snaps, "copies".to_string()).unwrap();

        assert_eq!(report.chunks_processed, COPIES);
        assert_eq!(report.new_chunks, 1, "a shared chunk must be counted as new once");
        assert_eq!(report.reused_chunks, COPIES - 1);

        let objects: Vec<_> = std::fs::read_dir(dir.join(".snapr/objects"))
            .unwrap()
            .map(|e| e.unwrap())
            .collect();
        assert_eq!(objects.len(), 1);
        let on_disk = objects[0].metadata().unwrap().len();
        assert_eq!(report.new_storage_bytes as u64, on_disk);
        assert_eq!(load_config().unwrap().get_total_storage_bytes(), on_disk);
    });
}
