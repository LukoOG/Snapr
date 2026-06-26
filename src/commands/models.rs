type FilePath = String;

#[derive(Debug)]
pub(super) struct DiffResult {
    pub added: Vec<FilePath>,
    pub modified: Vec<FilePath>,
    pub removed: Vec<FilePath>,
}

impl Default for DiffResult {
    fn default() -> Self {
        DiffResult {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
        }
    }
}