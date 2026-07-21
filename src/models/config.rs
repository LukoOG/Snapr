use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SnaprConfig {
    version: u32,
    pub current_snapshot: Option<u32>,
    pub repository_size: u64,
}

impl SnaprConfig {
    pub fn new() -> Self {
        SnaprConfig {
            version: 1,
            current_snapshot: None,
            repository_size: 0,
        }
    }

    pub fn update_repository_size(&mut self, size: u64) {
        self.repository_size += size;
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