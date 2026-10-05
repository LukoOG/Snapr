use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SnaprConfig {
    version: u32,
    current_snapshot: Option<u32>,
    total_storage_bytes: u64,
}

impl Default for SnaprConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl SnaprConfig {
    pub fn new() -> Self {
        SnaprConfig {
            version: 1,
            current_snapshot: None,
            total_storage_bytes: 0,
        }
    }

    pub fn get_total_storage_bytes(&self) -> u64 {
        self.total_storage_bytes
    }

    pub fn update_total_storage_bytes(&mut self, size: u64) {
        self.total_storage_bytes += size;
    }

    pub fn update_current_snapshot_id(&mut self, id: u32) {
        self.current_snapshot = Some(id)
    }

    pub fn set_current_snapshot_id(&mut self, id: u32) {
        self.current_snapshot = Some(id)
    }

    pub fn get_current_snapshot_id(&self) -> Option<u32> {
        self.current_snapshot
    }
}