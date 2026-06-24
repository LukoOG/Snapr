use crate::models::SnaprConfig;
use std::{fs, path::Path};

pub fn load_config() -> Result<SnaprConfig, Box<dyn std::error::Error>> {
    let file_path = ".snapr/config.json";
    if !Path::new(file_path).exists() {
        return Ok(SnaprConfig::new());
    }

    let config: SnaprConfig = serde_json::from_str(file_path)?;
    Ok(config)
}

pub fn save_config(config: &SnaprConfig) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(".snapr/config.json", json)?;
    Ok(())
}
