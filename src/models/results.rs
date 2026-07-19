use super::reports::FileStoreReport;

pub struct FileProcessResult {
    pub chunk_hashes: Vec<String>,
    pub report: FileStoreReport,
}

pub struct ChunkStoreResult {
    pub stored: bool,
    pub original_size: usize,
    pub compressed_size: usize,
    //To aid in builder pipeline
    pub hash: String,
}