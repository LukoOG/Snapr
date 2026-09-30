use std::collections::HashMap;

use super::models::DiffResult;
use crate::{models::{FileEntry, SnapshotFiles}, scoped_timer};

pub(super) fn compare_snapshots<F: SnapshotFiles, T: SnapshotFiles>(from: &F, to: &T) -> DiffResult {
    scoped_timer!("Comparing snapshots");
    let source = from
        .files()
        .iter()
        .map(|FileEntry { chunk_hashes, path }| (path.as_str(), chunk_hashes.as_slice()))
        .collect::<HashMap<_, _>>();
    let target = to
        .files()
        .iter()
        .map(|FileEntry { chunk_hashes, path }| (path.as_str(), chunk_hashes.as_slice()))
        .collect::<HashMap<_, _>>();

    let mut result = DiffResult::default();

    for (path, hash) in target.iter() {
        if let Some(old_hash) = source.get(path) {
            if hash != old_hash {
                result.modified.push((*path).to_owned());
            }
        } else {
            result.added.push((*path).to_owned())
        }
    }

    for path in source.keys() {
        if !target.contains_key(path) {
            result.removed.push((*path).to_owned())
        }
    }

    result.added.sort();
    result.modified.sort();
    result.removed.sort();

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileEntry, SnapshotFiles};

    struct FakeSnapshot(Vec<FileEntry>);
    impl SnapshotFiles for FakeSnapshot {
        fn files(&self) -> &[FileEntry] {
            &self.0
        }
    }

    fn entry(path: &str, hashes: &[&str]) -> FileEntry {
        FileEntry {
            path: path.to_string(),
            chunk_hashes: hashes.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn snap(entries: &[(&str, &[&str])]) -> FakeSnapshot {
        FakeSnapshot(entries.iter().map(|(p, h)| entry(p, h)).collect())
    }

    // Invariant 22: identical snapshots → all-empty DiffResult.
    #[test]
    fn identical_snapshots_produce_empty_diff() {
        let a = snap(&[("a.txt", &["h1"]), ("b.txt", &["h2"])]);
        let b = snap(&[("a.txt", &["h1"]), ("b.txt", &["h2"])]);
        let diff = compare_snapshots(&a, &b);
        assert!(diff.added.is_empty());
        assert!(diff.modified.is_empty());
        assert!(diff.removed.is_empty());
    }

    // Invariant 19: file in B not in A → added.
    #[test]
    fn file_in_new_not_in_old_is_added() {
        let old = snap(&[("a.txt", &["h1"])]);
        let new = snap(&[("a.txt", &["h1"]), ("new.txt", &["h2"])]);
        let diff = compare_snapshots(&old, &new);
        assert_eq!(diff.added, vec!["new.txt"]);
        assert!(diff.modified.is_empty());
        assert!(diff.removed.is_empty());
    }

    // Invariant 20: file in A not in B → removed.
    #[test]
    fn file_in_old_not_in_new_is_removed() {
        let old = snap(&[("a.txt", &["h1"]), ("gone.txt", &["h2"])]);
        let new = snap(&[("a.txt", &["h1"])]);
        let diff = compare_snapshots(&old, &new);
        assert!(diff.added.is_empty());
        assert!(diff.modified.is_empty());
        assert_eq!(diff.removed, vec!["gone.txt"]);
    }

    // Invariant 21: same path, different hashes → modified.
    #[test]
    fn same_path_different_hashes_is_modified() {
        let old = snap(&[("a.txt", &["old_hash"])]);
        let new = snap(&[("a.txt", &["new_hash"])]);
        let diff = compare_snapshots(&old, &new);
        assert!(diff.added.is_empty());
        assert_eq!(diff.modified, vec!["a.txt"]);
        assert!(diff.removed.is_empty());
    }

    // All three categories simultaneously.
    #[test]
    fn all_three_categories_simultaneously() {
        let old = snap(&[
            ("unchanged.txt", &["h1"]),
            ("modified.txt",  &["old"]),
            ("removed.txt",   &["hr"]),
        ]);
        let new = snap(&[
            ("unchanged.txt", &["h1"]),
            ("modified.txt",  &["new"]),
            ("added.txt",     &["ha"]),
        ]);
        let diff = compare_snapshots(&old, &new);
        assert_eq!(diff.added,    vec!["added.txt"]);
        assert_eq!(diff.modified, vec!["modified.txt"]);
        assert_eq!(diff.removed,  vec!["removed.txt"]);
    }

    // Invariant 23: path comparison is exact string equality.
    #[test]
    fn path_comparison_is_case_sensitive() {
        let old = snap(&[("File.txt", &["h1"])]);
        let new = snap(&[("file.txt", &["h1"])]);
        let diff = compare_snapshots(&old, &new);
        // "File.txt" removed, "file.txt" added — not treated as a rename/match.
        assert_eq!(diff.added,   vec!["file.txt"]);
        assert_eq!(diff.removed, vec!["File.txt"]);
        assert!(diff.modified.is_empty());
    }

    // Output lists are sorted.
    #[test]
    fn output_lists_are_sorted() {
        let old = snap(&[("z.txt", &["hz"]), ("m.txt", &["hm"])]);
        let new = snap(&[("a.txt", &["ha"]), ("b.txt", &["hb"])]);
        let diff = compare_snapshots(&old, &new);
        assert_eq!(diff.added,   vec!["a.txt", "b.txt"]);
        assert_eq!(diff.removed, vec!["m.txt", "z.txt"]);
    }

    // Multi-chunk files: modified only when chunk list differs.
    #[test]
    fn multi_chunk_file_modified_when_hashes_differ() {
        let old = snap(&[("big.bin", &["c1", "c2", "c3"])]);
        let new_same = snap(&[("big.bin", &["c1", "c2", "c3"])]);
        let new_diff = snap(&[("big.bin", &["c1", "c2", "c4"])]);

        let diff_same = compare_snapshots(&old, &new_same);
        assert!(diff_same.modified.is_empty());

        let diff_diff = compare_snapshots(&old, &new_diff);
        assert_eq!(diff_diff.modified, vec!["big.bin"]);
    }

    // Empty snapshots → empty diff.
    #[test]
    fn both_empty_snapshots() {
        let a = snap(&[]);
        let b = snap(&[]);
        let diff = compare_snapshots(&a, &b);
        assert!(diff.added.is_empty());
        assert!(diff.modified.is_empty());
        assert!(diff.removed.is_empty());
    }
}
