use super::results::*;

#[allow(unused)]

#[derive(Default)]
pub struct FileStoreReport {
    pub total_chunks: usize,
    pub new_chunks: usize,
    pub reused_chunks: usize,
    pub original_bytes: usize,
    pub new_storage_bytes: usize,
}

#[derive(Default)]
pub struct StoreReport {
    pub total_files: usize,

    pub total_chunks: usize,
    pub new_chunks: usize,
    pub reused_chunks: usize,

    pub original_bytes: usize,
    pub new_storage_bytes: usize,
}

impl FileStoreReport {
    pub fn record(&mut self, result: &ChunkStoreResult) {
        self.total_chunks += 1;
        self.original_bytes += result.original_size;

        if result.stored {
            self.new_chunks += 1;
            self.new_storage_bytes += result.compressed_size;
        } else {
            self.reused_chunks += 1;
        }
    }
}

impl StoreReport {
    pub fn merge(&mut self, file: &FileStoreReport) {
        self.total_files += 1;

        self.total_chunks += file.total_chunks;
        self.new_chunks += file.new_chunks;
        self.reused_chunks += file.reused_chunks;

        self.original_bytes += file.original_bytes;
        self.new_storage_bytes += file.new_storage_bytes;
    }

    #[inline]
    pub fn deduplication_ratio(&self) -> f64 {
        if self.total_chunks == 0 {
            return 0.0;
        }

        self.reused_chunks as f64 / self.total_chunks as f64
    }

    #[inline]
    pub fn compression_ratio(&self) -> f64 {
        if self.original_bytes > 0 {
            100.0 * (1.0 - self.new_storage_bytes as f64 / self.original_bytes as f64)
        } else {
            0.0
        }
    }
}
