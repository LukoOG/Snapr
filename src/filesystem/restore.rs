use crate::storage::read_chunk;
use std::{fs::{self, File}, io::Write, path::Path};

pub fn restore_file(path: &str, hashes: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    };
    let mut file = File::create(path)?;
    for hash in hashes {
        let chunk = read_chunk(hash)?;
        // Later: Turn this into streaming
        file.write_all(&chunk)?;
    }
    Ok(())
}
