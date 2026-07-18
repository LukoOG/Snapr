use std::fs::File;
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
    pub compressed: Vec<u8>,
    pub original_size: usize,
}

pub struct ChunkReader<R> {
    reader: BufReader<R>,
    chunk_size: usize,
    next_index: usize,
}

impl Chunk {
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl HashedChunk {
    pub fn from_chunk(chunk: Chunk, hash: String) -> Self {
        HashedChunk {
            index: chunk.index,
            hash,
            bytes: chunk.bytes,
        }
    }
}

impl <R: Read> ChunkReader<R> {
    pub fn new(reader: R, chunk_size: usize) -> Self {
        ChunkReader {
            reader: BufReader::new(reader),
            chunk_size,
            next_index: 0,
        }
    }

    pub fn next_chunk(&mut self) -> io::Result<Option<Chunk>> {
        let mut buffer = vec![0; self.chunk_size];
        let bytes_read = self.reader.read(&mut buffer)?;

        buffer.truncate(bytes_read);

        if bytes_read == 0 {
            Ok(None)
        } else {
            let chunk = Chunk {
                index: self.next_index,
                bytes: buffer.clone(),
            };
            self.next_index += 1;
            Ok(Some(chunk))
        }
    }
}