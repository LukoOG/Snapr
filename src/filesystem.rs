use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

use crate::hash::hash_file;
use crate::models::FileEntry;

fn should_skip(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str();

    match entry.file_type() {
        t if t.is_dir() => {
            matches!(
                name,
                Some(".git" | "target" | ".snapr")
            )
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

pub fn store_object(hash: &str, contents: &[u8]) -> Result<bool, Box<dyn Error>> {
    debug_assert_eq!(hash.len(), 64);
    let path = format!(".snapr/objects/{}", hash);
    let file_exists = Path::new(&path).exists();

    if file_exists {
        return Ok(false);
    }
    fs::write(path, contents)?;
    Ok(true)
}

pub fn build_entries() -> Result<Vec<FileEntry>, Box<dyn Error>> {
    let files = collect_files()?;
    let mut entries: Vec<FileEntry> = Vec::new();
    {
        for file in files {
            let (hash, contents) = hash_file(&file)?;
            store_object(&hash, &contents)?;
            entries.push(FileEntry {
                path: file.to_string_lossy().to_string(),
                hash,
            });
        }
    };
    Ok(entries)
}
