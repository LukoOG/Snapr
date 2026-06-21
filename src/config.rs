use std::fs;
use crate::models::SnaprConfig;

pub fn load_config() -> Result<SnaprConfig, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(".snapr/config.json")
        .unwrap_or_else(|_| "{}".to_string());

    let config: SnaprConfig = serde_json::from_str(&contents)
        .unwrap_or(SnaprConfig::new());

    Ok(config)
}

pub fn save_config(config: &SnaprConfig) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(".snapr/config.json", json)?;
    Ok(())
}