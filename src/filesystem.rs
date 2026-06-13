use std::{error::Error, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    if entry.depth() == 0 {
        return false
    }

    if entry.file_name() == "target" { //test
        return true
    }
    entry.file_type().is_dir()
        && entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
}

pub fn collect_files() -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let files = WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(Result::ok)
        .map(|e| e.into_path())
        .collect();
    Ok(files)
}
