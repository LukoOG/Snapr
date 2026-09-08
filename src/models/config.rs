use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SnaprConfig {
    version: u32,
    pub current_snapshot: Option<u32>,
    pub total_storage_bytes: u64,
}

impl SnaprConfig {
    pub fn new() -> Self {
        SnaprConfig {
            version: 1,
            current_snapshot: None,
            total_storage_bytes: 0,
        }
    }

    pub fn update_total_storage_bytes(&mut self, size: u64) {
        self.total_storage_bytes += size;
    }

    pub fn update_current_snapshot(&mut self) {
        match self.current_snapshot {
            None => self.current_snapshot = Some(1),
            Some(id) => self.current_snapshot = Some(1 + id),
        }
    }

    pub fn set_current_snapshot(&mut self, id: u32) {
        self.current_snapshot = Some(id)
    }
}