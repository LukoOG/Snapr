use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

fn should_skip(entry: &DirEntry) -> bool {
    entry.file_type().is_dir()
        && matches!(
            entry.file_name().to_str(),
            Some(".git" | "target" | ".snapr")
        )
}

pub fn collect_files() -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let files = WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| !should_skip(e))
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();
    Ok(files)
}

pub fn store_object(hash: &str, contents: &[u8]) -> Result<bool, Box<dyn Error>> {
    debug_assert_eq!(hash.len(), 64);
    let path = format!(".snapr/objects/{}", hash);
    let file_exists = Path::new(&path).exists();

    if file_exists {
       return Ok(false)
    }
    fs::write(path, contents)?;
    Ok(true)
}
