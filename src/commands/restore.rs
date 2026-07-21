use super::helpers::calculate_diff;
use crate::commands::models::RestoreOptions;
use crate::config::{load_config, save_config};
use crate::error::SnaprResult;
use crate::filesystem::restore_file;
use crate::models::{RestoreReport, Snapshot, WorkspaceSnapshot};
use crate::processing::build_entries;

use std::collections::HashMap;
use std::println;
use std::{fs};

pub fn handle_restore(
    snapshots: &[Snapshot],
    options: RestoreOptions,
) -> SnaprResult<RestoreReport> {
    let mut config = load_config()?;
    let RestoreOptions {
        snapshot_id,
        force,
        dry_run,
    } = options;

    //check if on current snapshot
    if config.current_snapshot == Some(snapshot_id) && !force {
        println!(
            "Already on snapshot {}.\nUse --force to restore anyway.",
            snapshot_id
        );
        std::process::exit(0);
        // return Ok(RestoreReport {
        //     snapshot_id,
        //     ..Default::default()
        // });
    }

    let target_snapshot = snapshots
        .iter()
        .find(|s| s.id == snapshot_id)
        .ok_or("Snapshot not found")?;
    let current_workspace = WorkspaceSnapshot::build(build_entries()?);

    let diff = calculate_diff(&current_workspace, target_snapshot);

    let restored = diff.added.len() + diff.modified.len();
    let removed = diff.removed.len();
    let skipped = target_snapshot.files.len() - restored;

    if dry_run {
        return Ok(RestoreReport {
            snapshot_id,
            restored_files: restored,
            removed_files: removed,
            skipped_files: skipped,
            restored_bytes: 0,
            dry_run: true,
        });
    }

    let target_map = target_snapshot
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.chunk_hashes.clone()))
        .collect::<HashMap<_, _>>();

    for path in &diff.removed {
        fs::remove_file(path)?;
    }

    for path in diff.added.iter().chain(diff.modified.iter()) {
        let hashes = target_map
            .get(path.as_str())
            .ok_or("Missing file in snapshot")?;
        restore_file(path, hashes)?;
    }

    //config
    config.set_current_snapshot(snapshot_id);
    save_config(&config)?;
    Ok(RestoreReport {
        snapshot_id,
        restored_files: restored,
        removed_files: removed,
        skipped_files: skipped,
        restored_bytes: 0, // This would need to be calculated if needed
        dry_run: false,
    })
}
