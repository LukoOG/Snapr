#[derive(Debug)]
pub(super) struct DiffResult {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
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