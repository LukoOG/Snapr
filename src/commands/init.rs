use std::error::Error;
use std::fs;
use std::path::{Path};

///Using try_exists
// pub fn handle_init() -> Result<(), Box<dyn Error>> {
//     let path = Path::new(".snapr");

//     match path.try_exists() {
//         Ok(true) => {
//             println!("snapr already initialized")
//         },
//         Ok(false) => {
//             fs::create_dir_all(".snapr/objects/")?;
//             fs::write(".snapr", "config.json")?;
//             fs::write(".snapr", "snapshot.json")?;
//         },
//         Err(e) => {
//             return Err(Box::new(e))
//         }

//     }
//     Ok(())
// }

pub fn handle_init() -> Result<(), Box<dyn Error>>{
    let path = Path::new(".snapr");

    if path.exists() {
        println!("Snapr already initialized!");
        return Ok(())
    }

    fs::create_dir_all(".snapr/objects")?;
    fs::write(".snapr/config.json", r#"{"version": 1}"#)?;
    fs::write(".snapr/snapshots.json", "[]")?;
    println!("Initialized snapr workspace");

    Ok(())
}