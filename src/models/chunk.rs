use std::io::{self, BufReader, Read};

pub const DEFAULT_CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4 MB

#[derive(Debug)]
pub struct Chunk {
    pub index: usize,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct HashedChunk {
    pub index: usize,
    pub hash: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct CompressedChunk {
    pub index: usize,
    pub hash: String,
    pub object_bytes: Vec<u8>,
    pub original_size: usize,
}

pub struct ChunkReader<R> {
    reader: BufReader<R>,
    chunk_size: usize,
    next_index: usize,
}

impl Chunk {
    pub fn new(index: usize, bytes: Vec<u8>) -> Self {
        Self { index, bytes }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl HashedChunk {
    pub fn from_chunk(chunk: Chunk, hash: String) -> Self {
        Self {
            index: chunk.index,
            hash,
            bytes: chunk.bytes,
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl CompressedChunk {
    pub fn len(&self) -> usize {
        self.object_bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.object_bytes.is_empty()
    }
}

impl<R: Read> ChunkReader<R> {
    pub fn new(reader: R, chunk_size: usize) -> Self {
        ChunkReader {
            reader: BufReader::new(reader),
            chunk_size,
            next_index: 0,
        }
    }

    pub fn next_chunk(&mut self) -> io::Result<Option<Chunk>> {
        if self.chunk_size == 0 {
            return Ok(None);
        }

        let mut buffer = vec![0; self.chunk_size];
        let mut bytes_read = 0;

        while bytes_read < buffer.len() {
            match self.reader.read(&mut buffer[bytes_read..])? {
                0 => break,
                read => bytes_read += read,
            }
        }

        buffer.truncate(bytes_read);

        if bytes_read == 0 {
            Ok(None)
        } else {
            let chunk = Chunk::new(self.next_index, buffer);
            self.next_index += 1;
            Ok(Some(chunk))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_chunks<R: std::io::Read>(reader: R, chunk_size: usize) -> Vec<Chunk> {
        let mut cr = ChunkReader::new(reader, chunk_size);
        let mut chunks = Vec::new();
        while let Some(chunk) = cr.next_chunk().expect("read error") {
            chunks.push(chunk);
        }
        chunks
    }

    // Invariant 7: Empty input → zero chunks.
    #[test]
    fn empty_reader_yields_no_chunks() {
        let chunks = collect_chunks(std::io::empty(), 8);
        assert!(chunks.is_empty());
    }

    // Chunk size of 0 → zero chunks (guard against infinite-loop risk).
    #[test]
    fn zero_chunk_size_yields_no_chunks() {
        let data: &[u8] = b"hello";
        let chunks = collect_chunks(data, 0);
        assert!(chunks.is_empty());
    }

    // Invariant 6: Input smaller than chunk_size → exactly one chunk with all bytes.
    #[test]
    fn input_smaller_than_chunk_size_yields_one_chunk() {
        let data: &[u8] = b"hello";
        let chunks = collect_chunks(data, 64);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].bytes, b"hello");
        assert_eq!(chunks[0].index, 0);
    }

    // Invariant 8: Input exactly equal to chunk_size → one full chunk.
    #[test]
    fn input_exactly_chunk_size_yields_one_full_chunk() {
        let data: Vec<u8> = (0u8..16).collect();
        let chunks = collect_chunks(data.as_slice(), 16);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].bytes.len(), 16);
        assert_eq!(chunks[0].bytes, data);
    }

    // Invariant 5: Multiple chunks reconstruct the original data.
    #[test]
    fn multiple_chunks_reconstruct_original() {
        let data: Vec<u8> = (0u8..=255).cycle().take(100).collect();
        let chunk_size = 16;
        let chunks = collect_chunks(data.as_slice(), chunk_size);

        // Concatenate all chunk bytes.
        let reconstructed: Vec<u8> = chunks.iter().flat_map(|c| c.bytes.iter().copied()).collect();
        assert_eq!(reconstructed, data);
    }

    // Chunk indices must be sequential starting from 0.
    #[test]
    fn chunk_indices_are_sequential() {
        let data: Vec<u8> = vec![0u8; 50];
        let chunks = collect_chunks(data.as_slice(), 16);
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i, "chunk index mismatch at position {i}");
        }
    }

    // Exact multiple of chunk_size → all chunks are full, last included.
    #[test]
    fn exact_multiple_of_chunk_size() {
        let data: Vec<u8> = vec![42u8; 32];
        let chunks = collect_chunks(data.as_slice(), 8);
        assert_eq!(chunks.len(), 4);
        for chunk in &chunks {
            assert_eq!(chunk.bytes.len(), 8);
        }
    }

    // Last chunk may be partial when data is not a multiple of chunk_size.
    #[test]
    fn last_chunk_is_partial_when_not_multiple() {
        let data: Vec<u8> = vec![1u8; 10];
        let chunks = collect_chunks(data.as_slice(), 3);
        // 10 bytes / 3 = 3 full + 1 partial
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[3].bytes.len(), 1);
    }
}
