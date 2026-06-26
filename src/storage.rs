use crate::{
    constants::{FLAG_NONE, HEADER_SIZE, MAGIC, VERSION_1},
    models::{CompressionType, Snapshot, StoreResult},
};
use std::{error::Error, fs, path::Path};
use zstd::{decode_all, encode_all};

pub fn load_snapshots() -> Result<Vec<Snapshot>, Box<dyn Error>> {
    let contents = fs::read_to_string(".snapr/snapshots.json").map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Snapr not initialized")
        } else {
            e
        }
    })?;
    let parsed = serde_json::from_str(&contents)?;
    Ok(parsed)
}

pub fn store_object(hash: &str, contents: &[u8]) -> Result<StoreResult, Box<dyn Error>> {
    if hash.len() != 64 {
        return Err("invalid sha256 hash".into());
    }
    let path = format!(".snapr/objects/{}", hash);

    if Path::new(&path).exists() {
        return Ok(StoreResult {
            stored: false,
            original_size: contents.len(),
            compressed_size: 0,
        });
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
    Ok(StoreResult {
        stored: true,
        original_size: contents.len(),
        compressed_size: compressed.len(),
    })
}

pub fn restore_object(path: &str, object_path: &str) -> Result<(), Box<dyn Error>> {
    let object = fs::read(object_path).map_err(|_| format!("Missing object: {}", object_path))?;
    if &object[..5] != MAGIC {
        return Err("invalid object".into());
    }

    let _version = object[5];
    let _flags = object[6];
    let _compression = object[7];

    let _original_size = u64::from_le_bytes(object[8..16].try_into()?);

    let compressed = &object[HEADER_SIZE..];
    let contents = decode_all(&compressed[..])?;

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;
    Ok(())
}
