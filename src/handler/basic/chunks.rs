use crate::acceptors::traits::Sender;
use crate::handler::traits::ServiceConnection;
use crate::http::streaming_parser::ChunkParser;

use log::error;

pub async fn forward<R, S>(
    id: u32,
    con: &mut R,
    sender: &mut S,
    buffer: &mut [u8],
    initial_data: usize,
) where
    R: ServiceConnection + Send,
    S: Sender + Send,
{
    let mut chunk_parser = ChunkParser::new();
    if initial_data > 0 {
        let (done, _left_over) = chunk_parser.block_parse(&buffer[..initial_data]);
        if done {
            let result = match chunk_parser.finish() {
                Some(r) => r,
                None => return,
            };
            let chunk_size = result.size();

            let mut out = Vec::with_capacity(result.size() + 32);
            result.serialize(&mut out);
            let out_length = out.len();
            sender.send(out, out_length).await;

            if chunk_size == 0 {
                return;
            }

            chunk_parser = ChunkParser::new();
        }
    }

    loop {
        match con.read(buffer).await {
            Ok(n) if n == 0 => {
                return;
            }
            Ok(n) => {
                let mut start = 0;
                let end = n;
                loop {
                    let (done, left_over) = chunk_parser.block_parse(&buffer[start..end]);
                    if done {
                        let result = match chunk_parser.finish() {
                            Some(r) => r,
                            None => return,
                        };
                        let chunk_size = result.size();

                        let mut out = Vec::with_capacity(result.size() + 32);
                        result.serialize(&mut out);
                        let out_length = out.len();
                        sender.send(out, out_length).await;

                        if chunk_size == 0 {
                            return;
                        }

                        chunk_parser = ChunkParser::new();
                    }
                    if left_over == 0 {
                        break;
                    }
                    start = end - left_over;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                error!("[{}] Reading from Connection: {}", id, e);
                return;
            }
        };
    }
}

#[cfg(test)]
use crate::acceptors::mocks::Sender as MockSender;
#[cfg(test)]
use crate::handler::mocks::ServiceConnection as MockServiceConnection;

#[tokio::test]
async fn valid_no_inital_data_one_chunk_without_final_empty_chunk() {
    let mut con = MockServiceConnection::new();
    con.add_chunk("9\r\nTest Data\r\n".as_bytes().to_vec());

    let mut sender = MockSender::new();
    let id = 0;
    let mut buffer = [0; 2048];
    let inital_data = 0;

    forward(id, &mut con, &mut sender, &mut buffer, inital_data).await;

    assert_eq!(
        vec!["9\r\nTest Data\r\n".as_bytes().to_vec()],
        sender.get_chunks()
    );
}
#[tokio::test]
async fn valid_no_inital_data_one_chunk_final_empty_chunk() {
    let mut con = MockServiceConnection::new();
    con.add_chunk("9\r\nTest Data\r\n".as_bytes().to_vec());
    con.add_chunk("0\r\n\r\n".as_bytes().to_vec());

    let mut sender = MockSender::new();
    let id = 0;
    let mut buffer = [0; 2048];
    let inital_data = 0;

    forward(id, &mut con, &mut sender, &mut buffer, inital_data).await;

    assert_eq!(
        vec![
            "9\r\nTest Data\r\n".as_bytes().to_vec(),
            "0\r\n\r\n".as_bytes().to_vec()
        ],
        sender.get_chunks()
    );
}

#[tokio::test]
async fn valid_with_inital_data_one_chunk() {
    let mut con = MockServiceConnection::new();
    con.add_chunk("9\r\nTest Data\r\n".as_bytes().to_vec());

    let mut sender = MockSender::new();
    let id = 0;
    let mut buffer = [0; 2048];
    let inital_data = 10;

    buffer[..10].clone_from_slice("5\r\nOther\r\n".as_bytes());

    forward(id, &mut con, &mut sender, &mut buffer, inital_data).await;

    assert_eq!(
        vec![
            "5\r\nOther\r\n".as_bytes().to_vec(),
            "9\r\nTest Data\r\n".as_bytes().to_vec()
        ],
        sender.get_chunks()
    );
}

#[tokio::test]
async fn valid_no_inital_data_one_chunk_with_final_empty_chunk_in_first_received() {
    let mut con = MockServiceConnection::new();
    con.add_chunk("9\r\nTest Data\r\n0\r\n\r\n".as_bytes().to_vec());

    let mut sender = MockSender::new();
    let id = 0;
    let mut buffer = [0; 2048];
    let inital_data = 0;

    forward(id, &mut con, &mut sender, &mut buffer, inital_data).await;

    assert_eq!(
        vec![
            "9\r\nTest Data\r\n".as_bytes().to_vec(),
            "0\r\n\r\n".as_bytes().to_vec()
        ],
        sender.get_chunks()
    );
}
