use crate::acceptors::traits::Sender as SenderTrait;

use async_trait::async_trait;

pub struct Sender {
    chunks: Vec<Vec<u8>>,
}

impl Sender {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    /// Returns a slice of all the Chunks that were
    /// written using this Sender
    pub fn get_chunks(&self) -> &[Vec<u8>] {
        &self.chunks
    }

    /// Combines all the Data from the different chunks
    /// into a single large Vector.
    pub fn get_combined_data(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for chunk in &self.chunks {
            result.extend_from_slice(&chunk);
        }

        result
    }
}

#[async_trait]
impl SenderTrait for Sender {
    async fn send(&mut self, data: Vec<u8>, _length: usize) {
        self.chunks.push(data);
    }
}

#[test]
fn create_new() {
    let tmp = Sender::new();
    assert_eq!(Vec::<Vec<u8>>::new(), tmp.chunks);
}

#[tokio::test]
async fn write_and_get_chunks() {
    let mut tmp = Sender::new();
    assert_eq!(Vec::<Vec<u8>>::new(), tmp.chunks);

    tmp.send(vec![0, 1, 2, 3], 4).await;

    assert_eq!(vec![vec![0, 1, 2, 3]].as_slice(), tmp.get_chunks());
    assert_eq!(vec![0, 1, 2, 3], tmp.get_combined_data());
}

#[tokio::test]
async fn multiple_writes_and_get_chunks() {
    let mut tmp = Sender::new();
    assert_eq!(Vec::<Vec<u8>>::new(), tmp.chunks);

    tmp.send(vec![0, 1, 2, 3], 4).await;
    tmp.send(vec![5, 6, 7, 8], 4).await;

    assert_eq!(
        vec![vec![0, 1, 2, 3], vec![5, 6, 7, 8]].as_slice(),
        tmp.get_chunks()
    );
    assert_eq!(vec![0, 1, 2, 3, 5, 6, 7, 8], tmp.get_combined_data());
}
