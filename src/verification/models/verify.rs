#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct FileVerificationKey {
    pub path: String,
    pub chunk_hashes: Vec<String>,
}