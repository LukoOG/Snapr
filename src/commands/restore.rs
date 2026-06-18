use crate::models::FileEntry;
use crate::models::Snapshot;

use std::error::Error;
use std::fs;
use std::path::Path;

pub fn handle_restore(snapshots: &[Snapshot], snapshot_id: u32) -> Result<(), Box<dyn Error>> {
    let snapshot = snapshots
        .iter()
        .find(|s| s.id == snapshot_id)
        .ok_or("Snapshot not found")?;

    for FileEntry { path, hash } in snapshot.files.iter() {
        let object_path = format!(".snapr/objects/{}", hash);

        if !Path::new(&object_path).exists() {
            return Err(format!("Missing object: {}", hash).into())
        }

        println!("{:?} {:?}", &object_path, path);
        let contents = fs::read_to_string(&object_path)?;

        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, contents)?
    }

    println!("Restored snapshot {}", snapshot_id);
    Ok(())
}
