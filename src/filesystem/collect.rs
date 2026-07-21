use walkdir::{DirEntry, WalkDir};
use std::path::PathBuf;
use std::error::Error;

use crate::error::SnaprResult;

fn should_skip(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str();

    match entry.file_type() {
        t if t.is_dir() => {
            matches!(name, Some(".git" | "target" | ".snapr"))
        }
        _ => {
            matches!(name, Some(".DS_Store"))
        }
    }
}

pub fn collect_files() -> SnaprResult<Vec<PathBuf>> {
    let files = WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| !should_skip(e))
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            e.into_path()
                .strip_prefix(".")
                .expect("path should be relative to current directory")
                .to_path_buf()
        })
        .collect();
    Ok(files)
}