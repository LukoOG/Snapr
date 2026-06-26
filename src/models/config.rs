use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SnaprConfig {
    version: u32,
    pub current_snapshot: Option<u32>,
}

impl SnaprConfig {
    pub fn new() -> Self {
        SnaprConfig {
            version: 1,
            current_snapshot: None,
        }
    }

    // pub fn get_current_snapshot(&self) -> u32 {
    //     self.current_snapshot.unwrap()
    // }

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