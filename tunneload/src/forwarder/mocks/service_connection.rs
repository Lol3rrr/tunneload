use crate::forwarder::{ServiceConnection as ServiceConnectionTrait, ServiceReader, ServiceWriter};

use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ServiceConnection {
    reader: ConnectionReader,
    writer: ConnectionWriter,
}

impl ServiceConnection {
    pub fn new() -> Self {
        Self {
            reader: ConnectionReader::new(),
            writer: ConnectionWriter::new(),
        }
    }

    /// Adds a new Chunk to the list of chunks
    /// that should be returned as Data
    pub fn add_chunk(&mut self, data: Vec<u8>) {
        self.reader.add_chunk(data);
    }

    pub fn get_write_chunks(&self) -> &[Vec<u8>] {
        self.writer.get_chunks()
    }
}

impl Default for ServiceConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ServiceConnectionTrait for ServiceConnection {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf).await
    }

    type ReadHalf = ConnectionReader;
    type WriteHalf = ConnectionWriter;

    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf).await
    }

    fn halves_owned(self) -> (Self::ReadHalf, Self::WriteHalf) {
        (self.reader, self.writer)
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionReader {
    chunks: Vec<Vec<u8>>,
}

impl ConnectionReader {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub fn add_chunk(&mut self, data: Vec<u8>) {
        self.chunks.push(data);
    }
}

#[async_trait]
impl ServiceReader for ConnectionReader {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let chunk = match self.chunks.first_mut() {
            Some(f) => f,
            None => {
                return Ok(0);
            }
        };

        let chunk_length = chunk.len();
        let buf_length = buf.len();

        if chunk_length > buf_length {
            for (index, tmp) in chunk.drain(0..buf_length).enumerate() {
                buf[index] = tmp;
            }
            Ok(buf_length)
        } else {
            buf[..chunk_length].clone_from_slice(&chunk[..chunk_length]);

            self.chunks.remove(0);

            Ok(chunk_length)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionWriter {
    chunks: Vec<Vec<u8>>,
}

impl ConnectionWriter {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub fn get_chunks(&self) -> &[Vec<u8>] {
        &self.chunks
    }
}

#[async_trait]
impl ServiceWriter for ConnectionWriter {
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.chunks.push(buf.to_vec());
        Ok(buf.len())
    }
}
