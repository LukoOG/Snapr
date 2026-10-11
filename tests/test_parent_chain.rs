/// Integration tests for the snapshot parent-chain invariant.
///
/// Invariant: every non-root snapshot has exactly one parent, and following
/// parent links from any snapshot must eventually terminate at the root
/// (the unique snapshot with parent_id == None).
///
/// These tests use the real save/restore pipeline so they verify that
/// parent_id is recorded correctly on disk, not just in memory.

mod helpers;

use helpers::{with_repo, write_file};
use snapr::commands::restore::handle_restore;
use snapr::commands::save::handle_save;
use snapr::commands::models::RestoreOptions;
use snapr::storage::load_snapshots;
use std::collections::HashSet;

// ── Shared validation helper (mirrors the unit-test version but works on
//    persisted Snapshot values loaded from disk) ────────────────────────────

fn walk_chain(snapshots: &[snapr::models::Snapshot], start_id: u32) -> Result<Vec<u32>, String> {
    let mut visited = HashSet::new();
    let mut chain = Vec::new();
    let mut current = start_id;
    loop {
        if !visited.insert(current) {
            return Err(format!("cycle at {current}"));
        }
        chain.push(current);
        let s = snapshots.iter().find(|s| s.id == current)
            .ok_or_else(|| format!("parent_id {current} not found"))?;
        match s.parent_id {
            None  => return Ok(chain),
            Some(p) => current = p,
        }
    }
}

fn assert_valid_chain(snapshots: &[snapr::models::Snapshot]) {
    let roots: Vec<u32> = snapshots.iter()
        .filter(|s| s.parent_id.is_none())
        .map(|s| s.id)
        .collect();
    assert_eq!(roots.len(), 1,
        "expected exactly one root, found: {roots:?}");

    let root_id = roots[0];
    for s in snapshots {
        if let Some(pid) = s.parent_id {
            assert!(
                snapshots.iter().any(|x| x.id == pid),
                "snapshot {} references parent_id {} which does not exist", s.id, pid
            );
        }
        let chain = walk_chain(snapshots, s.id)
            .expect("chain walk must not fail on valid data");
        assert_eq!(
            *chain.last().unwrap(), root_id,
            "chain from {} does not terminate at root {}; chain={:?}", s.id, root_id, chain
        );
    }
}

fn save(msg: &str) {
    let mut snaps = load_snapshots().unwrap();
    handle_save(&mut snaps, msg.to_string()).unwrap();
}

fn restore_snap(id: u32) {
    let snaps = load_snapshots().unwrap();
    handle_restore(&snaps, RestoreOptions {
        snapshot_id: id,
        force: true,
        dry_run: false,
    }).unwrap();
}

// ── Tests ─────────────────────────────────────────────────────────────────────

// The very first snapshot is the root: parent_id must be None.
#[test]
fn first_snapshot_is_root() {
    with_repo(|_| {
        write_file("a.txt", b"hello");
        save("initial");

        let snaps = load_snapshots().unwrap();
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].id, 1);
        assert!(
            snaps[0].parent_id.is_none(),
            "first snapshot must have no parent, got {:?}", snaps[0].parent_id
        );
    });
}

// Sequential saves: each snapshot's parent is the previous one.
#[test]
fn sequential_saves_form_linear_chain() {
    with_repo(|_| {
        write_file("a.txt", b"v1");
        save("s1");
        write_file("b.txt", b"v2 - longer content");
        save("s2");
        write_file("c.txt", b"v3 - even longer content here");
        save("s3");

        let snaps = load_snapshots().unwrap();
        assert_eq!(snaps.len(), 3);

        // Explicit parent chain: 1←None, 2←1, 3←2.
        let find = |id| snaps.iter().find(|s| s.id == id).unwrap();
        assert_eq!(find(1).parent_id, None);
        assert_eq!(find(2).parent_id, Some(1));
        assert_eq!(find(3).parent_id, Some(2));

        assert_valid_chain(&snaps);
    });
}

// After restoring to snapshot 1 and saving, the new snapshot must point to
// snapshot 1 as its parent — not to snapshot 2 (which is no longer current).
#[test]
fn save_after_restore_branches_from_restored_snapshot() {
    with_repo(|_| {
        write_file("f.txt", b"version one content");
        save("s1");
        write_file("f.txt", b"version two - longer content here");
        save("s2");

        // Current head is s2. Restore to s1.
        restore_snap(1);

        // Save again — new snapshot should branch from s1.
        write_file("g.txt", b"new branch content after restore");
        save("s3-branched");

        let snaps = load_snapshots().unwrap();
        assert_eq!(snaps.len(), 3);

        let s3 = snaps.iter().find(|s| s.id == 3).unwrap();
        assert_eq!(
            s3.parent_id,
            Some(1),
            "snapshot saved after restoring to s1 must have parent_id=1, got {:?}", s3.parent_id
        );
        assert_ne!(s3.parent_id, Some(2),
            "snapshot must NOT point to s2 when s1 was current at save time");

        assert_valid_chain(&snaps);
    });
}

// The scenario from the invariant description:
// save s1..s4 sequentially, restore to s1, then save s5.
// s5.parent must be s1, not s4.
#[test]
fn snapshot_5_branches_from_1_after_restore() {
    with_repo(|_| {
        write_file("data.txt", b"snapshot one data");
        save("snapshot 1");
        write_file("data.txt", b"snapshot two data - longer content");
        save("snapshot 2");
        write_file("data.txt", b"snapshot three data - even longer content here");
        save("snapshot 3");
        write_file("data.txt", b"snapshot four data - longest of all the saves");
        save("snapshot 4");

        let snaps = load_snapshots().unwrap();
        assert_eq!(snaps.len(), 4);
        assert_valid_chain(&snaps);

        // Restore to snapshot 1.
        restore_snap(1);

        // Save snapshot 5.
        write_file("branch.txt", b"new work on branch from snapshot one");
        save("snapshot 5 - branch from 1");

        let snaps = load_snapshots().unwrap();
        assert_eq!(snaps.len(), 5);

        let s5 = snaps.iter().find(|s| s.id == 5).unwrap();
        assert_eq!(
            s5.parent_id,
            Some(1),
            "s5.parent_id must be 1 (the restored snapshot), got {:?}", s5.parent_id
        );

        // Full chain validation: two branches hang off s1.
        // Chain from s5: 5→1.
        // Chain from s4: 4→3→2→1.
        assert_valid_chain(&snaps);
    });
}

// Dry-run restore must not affect the current_snapshot_id tracked in config,
// so a subsequent save still inherits the correct parent.
#[test]
fn save_after_dry_run_restore_uses_correct_parent() {
    with_repo(|_| {
        write_file("f.txt", b"original content here");
        save("s1");
        write_file("f.txt", b"modified content - different length");
        save("s2");

        // Dry-run restore to s1 — current_snapshot_id stays at 2.
        let snaps = load_snapshots().unwrap();
        handle_restore(&snaps, RestoreOptions {
            snapshot_id: 1,
            force: true,
            dry_run: true,
        }).unwrap();

        // Save now — parent must be s2 (dry-run did not change current).
        write_file("g.txt", b"saved after dry-run restore");
        save("s3");

        let snaps = load_snapshots().unwrap();
        let s3 = snaps.iter().find(|s| s.id == 3).unwrap();
        assert_eq!(
            s3.parent_id,
            Some(2),
            "after dry-run restore, save should parent to s2 (current unchanged), got {:?}",
            s3.parent_id
        );

        assert_valid_chain(&snaps);
    });
}

// The full parent chain is persisted correctly in snapshots.json — verify by
// reloading from disk rather than using the in-memory vector.
#[test]
fn parent_chain_survives_serialisation_roundtrip() {
    with_repo(|_| {
        write_file("x.txt", b"data one");
        save("s1");
        write_file("y.txt", b"data two - longer");
        save("s2");

        // Reload from disk.
        let snaps = load_snapshots().unwrap();
        assert_eq!(snaps.iter().find(|s| s.id == 1).unwrap().parent_id, None);
        assert_eq!(snaps.iter().find(|s| s.id == 2).unwrap().parent_id, Some(1));
        assert_valid_chain(&snaps);
    });
}
