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