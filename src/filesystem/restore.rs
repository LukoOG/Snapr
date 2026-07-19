use std::{fs::File, io::Write};
use crate::storage::restore_chunk;

pub fn restore_file(path: &str, hashes: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    for hash in hashes {
        let chunk = restore_chunk(hash)?;
        // Process the restored chunk
        file.write_all(&chunk)?;
    }
    Ok(())
}