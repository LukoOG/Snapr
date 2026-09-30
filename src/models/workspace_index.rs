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

    pub fn get_hashes(&self) -> &[String] {
        &self.chunk_hashes
    }

    pub fn into_hashes(self) -> Vec<String> {
        self.chunk_hashes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Invariant 13: serialise → deserialise round-trip is lossless.
    #[test]
    fn workspace_index_serde_roundtrip() {
        let mut index = WorkspaceIndex::new();
        index.insert(
            "src/main.rs".to_string(),
            IndexedFile::new(1700000000, 1024, vec!["abc".to_string(), "def".to_string()]),
        );
        let json = serde_json::to_string(&index).unwrap();
        let restored: WorkspaceIndex = serde_json::from_str(&json).unwrap();
        let file = restored.get("src/main.rs").unwrap();
        assert_eq!(file.size, 1024);
        assert_eq!(file.modified, 1700000000);
        assert_eq!(file.chunk_hashes, vec!["abc", "def"]);
    }

    // Empty index serialises to the canonical shape.
    #[test]
    fn empty_index_serde() {
        let index = WorkspaceIndex::new();
        let json = serde_json::to_string(&index).unwrap();
        let restored: WorkspaceIndex = serde_json::from_str(&json).unwrap();
        assert!(restored.is_empty());
    }

    // lookup returns Hit for a present key.
    #[test]
    fn lookup_hit() {
        let mut index = WorkspaceIndex::new();
        index.insert("a.txt".to_string(), IndexedFile::new(1, 10, vec![]));
        assert!(matches!(index.lookup("a.txt"), IndexLookup::Hit(_)));
    }

    // lookup returns Miss for an absent key.
    #[test]
    fn lookup_miss() {
        let index = WorkspaceIndex::new();
        assert!(matches!(index.lookup("missing.txt"), IndexLookup::Miss));
    }

    // remove deletes the entry.
    #[test]
    fn remove_deletes_entry() {
        let mut index = WorkspaceIndex::new();
        index.insert("a.txt".to_string(), IndexedFile::new(1, 10, vec![]));
        index.remove("a.txt");
        assert!(!index.contains("a.txt"));
    }

    // clear empties the index.
    #[test]
    fn clear_empties_index() {
        let mut index = WorkspaceIndex::new();
        index.insert("a.txt".to_string(), IndexedFile::new(1, 10, vec![]));
        index.clear();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    // IndexedFile::matches uses a real file's metadata.
    // Invariant 11 & 12 — covered here with controlled metadata via tempfile.
    #[test]
    fn indexed_file_matches_uses_mtime_and_size() {
        use std::fs;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, b"hello").unwrap();

        let meta = fs::metadata(&path).unwrap();
        let mtime = meta
            .modified()
            .ok()
            .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Matching entry: same mtime + same size.
        let matching = IndexedFile::new(mtime, meta.len(), vec![]);
        assert!(matching.matches(&meta), "should match when mtime and size are equal");

        // Wrong size → no match.
        let wrong_size = IndexedFile::new(mtime, meta.len() + 1, vec![]);
        assert!(!wrong_size.matches(&meta));

        // Wrong mtime → no match.
        let wrong_mtime = IndexedFile::new(mtime.wrapping_add(1), meta.len(), vec![]);
        assert!(!wrong_mtime.matches(&meta));
    }
}
