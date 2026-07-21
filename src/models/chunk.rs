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
