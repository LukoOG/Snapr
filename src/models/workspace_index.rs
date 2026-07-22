use std::{collections::HashMap, fs::Metadata};

use serde::{Deserialize, Serialize};

type FilePath = String;
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WorkspaceIndex {
    pub files: HashMap<FilePath, IndexedFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexedFile {
    pub modified: u64,
    pub size: u64,
    pub chunk_hashes: Vec<String>,
}

pub enum IndexLookup<'a> {
    Hit(&'a IndexedFile),
    Miss,
}

impl WorkspaceIndex {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn lookup(&self, path: &str) -> IndexLookup<'_> {
        match self.files.get(path) {
            Some(file) => IndexLookup::Hit(file),
            None => IndexLookup::Miss,
        }
    }

    pub fn insert(&mut self, path: String, file: IndexedFile) {
        self.files.insert(path, file);
    }

    pub fn remove(&mut self, path: &str) {
        self.files.remove(path);
    }

    pub fn clear(&mut self) {
        self.files.clear();
    }

    pub fn contains(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    pub fn get(&self, path: &str) -> Option<&IndexedFile> {
        self.files.get(path)
    }

    pub fn get_mut(&mut self, path: &str) -> Option<&mut IndexedFile> {
        self.files.get_mut(path)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &IndexedFile)> {
        self.files.iter()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl IndexedFile {
    pub fn new(modified: u64, size: u64, chunk_hashes: Vec<String>) -> Self {
        Self {
            modified,
            size,
            chunk_hashes,
        }
    }

    pub fn matches(&self, metadata: &Metadata) -> bool {
        let modified = metadata
            .modified()
            .ok()
            .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.modified == modified && self.size == metadata.len()
    }

    pub fn chunk_hashes(&self) -> &[String] {
        &self.chunk_hashes
    }

    pub fn into_hashes(self) -> Vec<String> {
        self.chunk_hashes
    }
}
