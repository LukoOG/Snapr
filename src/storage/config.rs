use crate::{constants::CONFIG_FILE, error::SnaprResult, models::SnaprConfig};
use std::{fs, path::Path};

pub fn load_config() -> SnaprResult<SnaprConfig> {
    let file_path = CONFIG_FILE;
    if !Path::new(file_path).exists() {
        return Ok(SnaprConfig::new());
    }

    let contents = fs::read_to_string(file_path)?;
    let config: SnaprConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

pub fn save_config(config: &SnaprConfig) -> SnaprResult<()> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(".snapr/config.json", json)?;
    Ok(())
}
