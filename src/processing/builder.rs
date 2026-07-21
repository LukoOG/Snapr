use rayon::prelude::*;

use crate::error::SnaprResult;
use crate::filesystem::collect::collect_files;
use crate::models::FileEntry;
use crate::models::WorkspaceStoreReport;
use crate::processing::planner::EntryMode;
use crate::processing::planner::build_entry;
use crate::storage::workspace_index::load_workspace_index;
use crate::storage::workspace_index::save_workspace_index;

pub fn build_entries() -> SnaprResult<Vec<FileEntry>> {
    let mut index = load_workspace_index()?;
    let results = collect_files()?
        .par_iter()
        .map(|file| build_entry(file, EntryMode::Hash, &index))
        .collect::<SnaprResult<Vec<_>>>()?;

    let mut entries: Vec<FileEntry> = Vec::new();
    {
        for result in results {
            index.insert(result.path, result.cache_entry);
            entries.push(result.entry);
        }
    };
    save_workspace_index(&index)?;
    Ok(entries)
}

pub fn build_snapshot_entries() -> SnaprResult<(Vec<FileEntry>, WorkspaceStoreReport)> {
    let mut index = load_workspace_index()?;

    let mut entries: Vec<FileEntry> = Vec::new();
    let mut report = WorkspaceStoreReport::default();
    let results = collect_files()?
        .par_iter()
        .map(|file| build_entry(file, EntryMode::Store, &index))
        .collect::<SnaprResult<Vec<_>>>()?;
    {
        for result in results {
            index.insert(result.path, result.cache_entry);

            report.merge(&result.report);
            entries.push(result.entry);
        }
    };
    save_workspace_index(&index)?;
    Ok((entries, report))
}
