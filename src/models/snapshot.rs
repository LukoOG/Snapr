use serde::{Deserialize, Serialize};

use super::SnapshotStats;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub chunk_hashes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: u32,
    pub parent_id: Option<u32>,
    pub message: String,
    pub created_at: u64,
    pub files: Vec<FileEntry>,

    //metadata
    pub stats: SnapshotStats,
}

#[derive(Debug)]
pub struct WorkspaceSnapshot {
    pub files: Vec<FileEntry>,
}

pub trait SnapshotFiles {
    fn files(&self) -> &[FileEntry];
}

impl SnapshotFiles for Snapshot {
    fn files(&self) -> &[FileEntry] {
        &self.files
    }
}

impl WorkspaceSnapshot {
    pub fn build(entries: Vec<FileEntry>) -> Self {
        Self { files: entries }
    }
}

impl SnapshotFiles for WorkspaceSnapshot {
    fn files(&self) -> &[FileEntry] {
        &self.files
    }
}

#[cfg(test)]
mod parent_chain_tests {
    use super::*;

    // ── Helpers ────────────────────────────────────────────────────────────────

    fn snap(id: u32, parent_id: Option<u32>) -> Snapshot {
        Snapshot {
            id,
            parent_id,
            message: format!("snapshot {id}"),
            created_at: 0,
            files: vec![],
            stats: crate::models::SnapshotStats::default(),
        }
    }

    // Walk the parent chain from `start_id` and return the ordered list of IDs
    // visited, or an error if the chain is broken or cycles.
    fn walk_chain(snapshots: &[Snapshot], start_id: u32) -> Result<Vec<u32>, String> {
        let mut visited = std::collections::HashSet::new();
        let mut chain = Vec::new();
        let mut current = start_id;

        loop {
            if !visited.insert(current) {
                return Err(format!("cycle detected at id {current}"));
            }
            chain.push(current);
            let snap = snapshots.iter().find(|s| s.id == current)
                .ok_or_else(|| format!("parent_id {current} not found in snapshot list"))?;
            match snap.parent_id {
                None => return Ok(chain),   // reached the root
                Some(parent) => current = parent,
            }
        }
    }

    // Validate the full invariant for a slice of snapshots:
    //   1. Exactly one root (parent_id == None).
    //   2. Every non-root has a parent that exists.
    //   3. Following any chain terminates at the root.
    //   4. No cycles.
    fn assert_valid_chain(snapshots: &[Snapshot]) {
        let roots: Vec<_> = snapshots.iter().filter(|s| s.parent_id.is_none()).collect();
        assert_eq!(roots.len(), 1, "expected exactly one root snapshot, found: {:?}",
            roots.iter().map(|s| s.id).collect::<Vec<_>>());

        let root_id = roots[0].id;

        for snap in snapshots {
            // Every parent reference must resolve.
            if let Some(pid) = snap.parent_id {
                assert!(
                    snapshots.iter().any(|s| s.id == pid),
                    "snapshot {} has parent_id {} which does not exist", snap.id, pid
                );
            }
            // Every chain reaches the root.
            let chain = walk_chain(snapshots, snap.id)
                .expect("chain walk should not fail for valid snapshots");
            assert_eq!(
                *chain.last().unwrap(), root_id,
                "chain from snapshot {} does not terminate at root {}; chain={:?}",
                snap.id, root_id, chain
            );
        }
    }

    // ── Unit tests ─────────────────────────────────────────────────────────────

    // Root snapshot has no parent.
    #[test]
    fn root_snapshot_has_no_parent() {
        let s = snap(1, None);
        assert!(s.parent_id.is_none());
    }

    // A sequential chain 1 → 2 → 3 is valid.
    #[test]
    fn sequential_chain_is_valid() {
        let snaps = vec![snap(1, None), snap(2, Some(1)), snap(3, Some(2))];
        assert_valid_chain(&snaps);
    }

    // A branch: after restoring to 1 and saving, snapshot 3 branches from 1.
    // The chain 3→1 is valid; there is still exactly one root.
    #[test]
    fn branch_after_restore_is_valid() {
        // s1 (root) → s2, and s1 → s3 (saved after restoring to s1)
        let snaps = vec![snap(1, None), snap(2, Some(1)), snap(3, Some(1))];
        assert_valid_chain(&snaps);
    }

    // walk_chain on a single root returns [root_id].
    #[test]
    fn walk_chain_single_root() {
        let snaps = vec![snap(1, None)];
        assert_eq!(walk_chain(&snaps, 1).unwrap(), vec![1]);
    }

    // walk_chain returns ids in order from start to root.
    #[test]
    fn walk_chain_returns_path_to_root() {
        let snaps = vec![snap(1, None), snap(2, Some(1)), snap(3, Some(2))];
        assert_eq!(walk_chain(&snaps, 3).unwrap(), vec![3, 2, 1]);
        assert_eq!(walk_chain(&snaps, 2).unwrap(), vec![2, 1]);
        assert_eq!(walk_chain(&snaps, 1).unwrap(), vec![1]);
    }

    // walk_chain detects a broken parent reference.
    #[test]
    fn walk_chain_broken_parent_returns_error() {
        let snaps = vec![snap(2, Some(99))]; // parent 99 does not exist
        let result = walk_chain(&snaps, 2);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // A snapshot may not be its own parent (degenerate cycle).
    #[test]
    fn walk_chain_self_referential_cycle_returns_error() {
        let snaps = vec![snap(1, Some(1))];
        let result = walk_chain(&snaps, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cycle"));
    }

    // A two-node cycle is detected.
    #[test]
    fn walk_chain_two_node_cycle_returns_error() {
        let snaps = vec![snap(1, Some(2)), snap(2, Some(1))];
        let result = walk_chain(&snaps, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cycle"));
    }

    // Multiple roots → assert_valid_chain reports the violation.
    #[test]
    #[should_panic(expected = "exactly one root")]
    fn two_roots_fails_validation() {
        let snaps = vec![snap(1, None), snap(2, None)];
        assert_valid_chain(&snaps);
    }

    // Broken parent reference → assert_valid_chain reports the violation.
    #[test]
    #[should_panic(expected = "does not exist")]
    fn missing_parent_fails_validation() {
        let snaps = vec![snap(1, None), snap(2, Some(99))];
        assert_valid_chain(&snaps);
    }

    // IDs are non-sequential but the chain is still valid.
    #[test]
    fn non_sequential_ids_chain_is_valid() {
        // Save 1,2,3 → restore to 1 → save 5 (max+1=4, but let's test with 5).
        // parent of 5 is 1 (current at time of save after restore).
        let snaps = vec![
            snap(1, None),
            snap(2, Some(1)),
            snap(3, Some(2)),
            snap(4, Some(1)),  // saved after restoring to 1
        ];
        assert_valid_chain(&snaps);
    }
}
