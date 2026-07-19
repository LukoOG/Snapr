use crate::storage::restore_chunk;
use std::{fs::{self, File}, io::Write, path::Path};

pub fn restore_file(path: &str, hashes: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    };
    let mut file = File::create(path)?;
    for hash in hashes {
        let chunk = restore_chunk(hash)?;
        // Process the restored chunk
        file.write_all(&chunk)?;
    }
    Ok(())
}
