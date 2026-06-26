pub struct ObjectStoreResult {}

pub struct StoreResult {
    pub stored: bool,
    pub original_size: usize,
    pub compressed_size: usize,
}

#[derive(Default)]
pub struct StoreReport {
    pub total_files: usize,
    pub new_objects: usize,
    pub reused_objects: usize,
    pub original_bytes: usize,
    pub compressed_bytes: usize,
}

impl StoreReport {
    pub fn record(&mut self, record: &StoreResult) {
        self.total_files += 1;
        self.original_bytes += record.original_size;

        if record.stored {
            self.new_objects += 1;
            self.compressed_bytes += record.compressed_size;
        } else {
            self.reused_objects += 1;
        }
    }
}
