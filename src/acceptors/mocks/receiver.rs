use crate::acceptors::traits::Receiver as ReceiverTrait;

use async_trait::async_trait;

#[derive(Debug)]
pub struct Receiver {
    chunks: Vec<Vec<u8>>,
}

impl Receiver {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    /// Adds a new chunk to the end of the internal
    /// Chunk logs
    pub fn add_chunk(&mut self, n_chunk: Vec<u8>) {
        self.chunks.push(n_chunk);
    }
}

#[async_trait]
impl ReceiverTrait for Receiver {
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
            drop(chunk);

            self.chunks.remove(0);

            Ok(chunk_length)
        }
    }
}

#[test]
fn create_and_set_chunks() {
    let mut tmp = Receiver::new();
    assert_eq!(Vec::<Vec<u8>>::new(), tmp.chunks);

    tmp.add_chunk(Vec::new());
    assert_eq!(vec![Vec::<u8>::new()], tmp.chunks);
}

#[tokio::test]
async fn read_no_chunk() {
    let mut tmp = Receiver::new();

    let mut buffer = [0; 4];
    assert_eq!(0, tmp.read(&mut buffer).await.unwrap());

    assert_eq!(Vec::<Vec<u8>>::new(), tmp.chunks);
    assert_eq!([0, 0, 0, 0], buffer);
}

#[tokio::test]
async fn read_one_entire_chunk() {
    let mut tmp = Receiver::new();

    let n_chunk = vec![0, 1, 2];
    tmp.add_chunk(n_chunk.clone());

    let mut buffer = [0; 4];
    assert_eq!(3, tmp.read(&mut buffer).await.unwrap());

    assert_eq!(Vec::<Vec<u8>>::new(), tmp.chunks);
    assert_eq!([0, 1, 2, 0], buffer);
}

#[tokio::test]
async fn read_half_chunk() {
    let mut tmp = Receiver::new();

    let n_chunk = vec![0, 1, 2, 3, 4, 5];
    tmp.add_chunk(n_chunk.clone());

    let mut buffer = [0; 3];
    assert_eq!(3, tmp.read(&mut buffer).await.unwrap());

    assert_eq!(vec![vec![3, 4, 5]], tmp.chunks);
    assert_eq!([0, 1, 2], buffer);
}

#[tokio::test]
async fn read_chunk_over_multiple_reads() {
    let mut tmp = Receiver::new();

    let n_chunk = vec![0, 1, 2, 3, 4, 5];
    tmp.add_chunk(n_chunk.clone());

    let mut buffer = [0; 2];
    assert_eq!(2, tmp.read(&mut buffer).await.unwrap());
    assert_eq!(vec![vec![2, 3, 4, 5]], tmp.chunks);
    assert_eq!([0, 1], buffer);

    assert_eq!(2, tmp.read(&mut buffer).await.unwrap());
    assert_eq!(vec![vec![4, 5]], tmp.chunks);
    assert_eq!([2, 3], buffer);

    assert_eq!(2, tmp.read(&mut buffer).await.unwrap());
    assert_eq!(Vec::<Vec<u8>>::new(), tmp.chunks);
    assert_eq!([4, 5], buffer);
}
