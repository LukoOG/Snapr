use crate::{error::SnaprResult, storage::read_chunk};
use std::{fs::{self, File}, io::Write, path::Path};

pub fn restore_file(path: &str, hashes: &[String]) -> SnaprResult<u64> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    };
    let mut file = File::create(path)?;
    let mut restored = 0_u64;
    for hash in hashes {
        let chunk = read_chunk(hash)?;
        restored += chunk.len() as u64;
        // Later: Turn this into streaming
        file.write_all(&chunk)?;
    }
    Ok(restored)
}
