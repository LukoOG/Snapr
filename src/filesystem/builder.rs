use crate::filesystem::{
    collect::collect_files,
    hash::{hash_and_get_contents, hash_file_bytes},
};
use crate::models::{FileEntry, StoreReport};
use crate::storage::store_object;
use std::{error::Error, fs};

pub fn build_entries() -> Result<Vec<FileEntry>, Box<dyn Error>> {
    let files = collect_files()?;
    let mut entries: Vec<FileEntry> = Vec::new();
    {
        for file in files {
            let contents = fs::read(&file)?;
            let hash = hash_file_bytes(&contents)?;
            entries.push(FileEntry::build(file.to_string_lossy().to_string(), hash));
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
