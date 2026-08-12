use super::results::*;
use thiserror::Error;

#[allow(unused)]

#[derive(Debug, Error)]
pub enum VerifyIssue {
    #[error("missing chunk: {hash}")]
    MissingChunk {
        hash: String,
    },

    #[error("invalid header for chunk: {hash}")]
    InvalidHeader {
        hash: String,
    },

    #[error("unsupported compression ({compression}) for chunk: {hash}")]
    UnsupportedCompression {
        hash: String,
        compression: u8,
    },

    #[error("Chunk contents do not satify metadata: {hash}")]
    CorruptedChunk {
        hash: String,
    },

    #[error("Object successfully decompressed but got {expected} instead of {actual}")]
    HashMismatch {
        expected: String,
        actual: String,
    },
}

#[derive(Default)]
pub struct FileStoreReport {
    pub total_chunks: usize,
    pub new_chunks: usize,
    pub reused_chunks: usize,
    pub original_bytes: usize,
    pub new_storage_bytes: usize,
}

#[derive(Default)]
pub struct WorkspaceStoreReport {
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

impl WorkspaceStoreReport {
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

#[derive(Default)]
pub struct RestoreReport {
    pub snapshot_id: u32,

    pub restored_files: usize,
    pub removed_files: usize,
    pub skipped_files: usize,

    pub restored_bytes: u64,
    pub dry_run: bool,
}

#[derive(Default)]

pub struct FileVerifyReport {
    pub chunks_checked: usize,
    pub bytes_verified: u64,
}

#[derive(Default)]
pub struct SnapshotVerifyReport {
    pub files_checked: usize,
    pub chunks_checked: usize,
    pub bytes_verified: u64,
}

#[derive(Default)]
pub struct VerifyReport {
    pub snapshots_checked: usize,

    pub files_checked: usize,

    pub chunks_checked: usize,
    pub chunks_verified: usize,
    pub chunks_referenced: usize,

    pub bytes_verified: u64,

    pub issues: Vec<VerifyIssue>,
}

impl FileVerifyReport {
    pub fn record(&mut self, result: &ChunkVerifyResult) {
        self.chunks_checked += 1;
        self.bytes_verified += result.original_size;
    }
}

impl SnapshotVerifyReport {
    pub fn merge(&mut self, report: &FileVerifyReport) {
        self.files_checked += 1;
        self.bytes_verified += report.bytes_verified;
        self.chunks_checked += report.chunks_checked
    }
}