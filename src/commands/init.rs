use std::error::Error;
use std::fs;
use std::path::Path;

pub fn handle_init() -> Result<(), Box<dyn Error>> {
    let path = Path::new(".snapr");

    if path.exists() {
        println!("Snapr already initialized!");
        return Ok(());
    }

    fs::create_dir_all(".snapr/objects")?;
    fs::write(
        ".snapr/config.json",
        r#"{"version": 1,"repository_size":0}"#,
    )?;
    fs::write(".snapr/snapshots.json", "[]")?;
    println!("Initialized snapr workspace");

    Ok(())
}
