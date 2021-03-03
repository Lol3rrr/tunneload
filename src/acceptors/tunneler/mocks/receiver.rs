use tunneler_core::{
    client::Receiver as ReceiverTrait,
    message::{Message, MessageHeader, MessageType},
    streams::error::RecvError,
};

use async_trait::async_trait;

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
    async fn recv_msg(&mut self) -> Result<Message, RecvError> {
        if self.chunks.len() == 0 {
            let msg = Message::new(MessageHeader::new(0, MessageType::EOF, 0), vec![]);
            return Ok(msg);
        }

        let chunk = self.chunks.remove(0);
        let chunk_length = chunk.len();
        let msg = Message::new(
            MessageHeader::new(0, MessageType::Data, chunk_length as u64),
            chunk,
        );

        Ok(msg)
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
async fn read_one_chunk() {
    let mut tmp = Receiver::new();

    tmp.add_chunk(vec![0, 1, 2, 3, 4]);
    assert_eq!(vec![vec![0, 1, 2, 3, 4]], tmp.chunks);

    let msg = tmp.recv_msg().await;
    assert_eq!(true, msg.is_ok());
    assert_eq!(&vec![0, 1, 2, 3, 4], msg.unwrap().get_data());

    let last_msg = tmp.recv_msg().await;
    assert_eq!(true, last_msg.is_ok());
    assert_eq!(&MessageType::EOF, last_msg.unwrap().get_header().get_kind());
}

#[tokio::test]
async fn read_two_chunks() {
    let mut tmp = Receiver::new();

    tmp.add_chunk(vec![0, 1, 2, 3, 4]);
    tmp.add_chunk(vec![5, 6, 7, 8, 9]);
    assert_eq!(vec![vec![0, 1, 2, 3, 4], vec![5, 6, 7, 8, 9]], tmp.chunks);

    let msg_1 = tmp.recv_msg().await;
    assert_eq!(true, msg_1.is_ok());
    assert_eq!(&vec![0, 1, 2, 3, 4], msg_1.unwrap().get_data());

    let msg_2 = tmp.recv_msg().await;
    assert_eq!(true, msg_2.is_ok());
    assert_eq!(&vec![5, 6, 7, 8, 9], msg_2.unwrap().get_data());

    let last_msg = tmp.recv_msg().await;
    assert_eq!(true, last_msg.is_ok());
    assert_eq!(&MessageType::EOF, last_msg.unwrap().get_header().get_kind());
}
