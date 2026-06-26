use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};
use zstd::{decode_all, stream::encode_all};

use crate::models::FileEntry;
use crate::{
    constants::{FLAG_NONE, HEADER_SIZE, MAGIC, VERSION_1},
    hash::hash_and_get_contents,
    models::CompressionType,
};

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

pub fn store_object(hash: &str, contents: &[u8]) -> Result<bool, Box<dyn Error>> {
    if hash.len() != 64 {
        return Err("invalid sha256 hash".into());
    }
    let path = format!(".snapr/objects/{}", hash);

    if Path::new(&path).exists() {
        return Ok(false);
    }
    let compressed = encode_all(contents, 3)?;
    let mut object = Vec::<u8>::with_capacity(HEADER_SIZE + compressed.len());
    object.extend_from_slice(MAGIC);
    object.push(VERSION_1);
    object.push(FLAG_NONE);
    object.push(CompressionType::Zstd as u8);
    object.extend_from_slice(&(contents.len()).to_le_bytes());
    object.extend_from_slice(&compressed);

    fs::write(path, object)?;
    Ok(true)
}

pub fn build_entries() -> Result<Vec<FileEntry>, Box<dyn Error>> {
    let files = collect_files()?;
    let mut entries: Vec<FileEntry> = Vec::new();
    {
        for file in files {
            let (hash, contents) = hash_and_get_contents(&file)?;
            store_object(&hash, &contents)?;
            entries.push(FileEntry {
                path: file.to_string_lossy().to_string(),
                hash,
            });
        }
    };
    Ok(entries)
}

pub fn restore_file(path: &str, object_path: &str) -> Result<(), Box<dyn Error>> {
    let object =
        fs::read(object_path).map_err(|_| format!("Missing object: {}", object_path))?;
    if &object[..5] != MAGIC {
        return Err("invalid object".into());
    }

    let version = object[5];
    let flags = object[6];
    let compression = object[7];

    let original_size = u64::from_le_bytes(object[8..16].try_into()?);

    let compressed = &object[HEADER_SIZE..];
    let contents = decode_all(&compressed[..])?;

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;
    Ok(())
}
