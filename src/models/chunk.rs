use std::fs::File;
use std::io::{self, BufReader, Read};

pub const DEFAULT_CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4 MB

#[derive(Debug)]
pub struct Chunk {
    pub index: usize,
    pub bytes: Vec<u8>,
}

pub struct ChunkReader<R> {
    reader: BufReader<R>,
    chunk_size: usize,
}

impl Chunk {
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl <R: Read> ChunkReader<R> {
    pub fn new(reader: R, chunk_size: usize) -> Self {
        ChunkReader {
            reader: BufReader::new(reader),
            chunk_size,
        }
    }

    pub fn next_chunk(&mut self) -> io::Result<Option<Vec<u8>>> {
        let mut buffer = vec![0; self.chunk_size];
        let bytes_read = self.reader.read(&mut buffer)?;

        buffer.truncate(bytes_read);

        if bytes_read == 0 {
            Ok(None)
        } else {
            Ok(Some(buffer))
        }
    }
}