use std::{error::Error, fs, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::hash::{hash_and_get_contents, hash_file_bytes};
use crate::models::{FileEntry, StoreReport};
use crate::storage::store_object;

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

pub fn collect_files() -> Result<Vec<PathBuf>, Box<dyn Error>> {
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

pub fn build_entries() -> Result<Vec<FileEntry>, Box<dyn Error>> {
    let files = collect_files()?;
    let mut entries: Vec<FileEntry> = Vec::new();
    {
        for file in files {
            let contents = fs::read(&file)?;
            let hash = hash_file_bytes(&contents)?;
            entries.push(FileEntry {
                path: file.to_string_lossy().to_string(),
                hash,
            });
        }
    };
    Ok(entries)
}

pub fn build_and_store_entries() -> Result<(Vec<FileEntry>, StoreReport), Box<dyn Error>> {
    let files = collect_files()?;
    let mut entries = Vec::new();
    let mut report = StoreReport::default();
    {
        for file in files {
            let (hash, contents) = hash_and_get_contents(&file)?;
            let result = store_object(&hash, &contents)?;
            report.record(&result);
            entries.push(FileEntry::build(file.to_string_lossy().to_string(), hash));
        }
    }

    Ok((entries, report))
}
