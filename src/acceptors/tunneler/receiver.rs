use crate::acceptors::traits::Receiver as ReceiverTrait;

use async_trait::async_trait;

pub struct Receiver<R>
where
    R: tunneler_core::client::Receiver + Send + Sync,
{
    reader: R,
    buffer: Vec<u8>,
}

impl<R> Receiver<R>
where
    R: tunneler_core::client::Receiver + Send + Sync,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl<R> ReceiverTrait for Receiver<R>
where
    R: tunneler_core::client::Receiver + Send + Sync,
{
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.buffer.is_empty() {
            match self.reader.recv_msg().await {
                Ok(msg) => {
                    if msg.is_eof() {
                        return Ok(0);
                    }

                    let data = msg.get_data();
                    let data_len = data.len();
                    let buf_len = buf.len();

                    let read_len = std::cmp::min(data_len, buf_len);
                    buf[..read_len].clone_from_slice(&data[..read_len]);

                    if read_len < data_len {
                        let to_store = data_len - read_len;
                        self.buffer.reserve(to_store);
                        self.buffer.extend_from_slice(&data[read_len..]);
                    }

                    Ok(read_len)
                }
                Err(_) => Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe)),
            }
        } else {
            let out_len = buf.len();
            let buffer_len = self.buffer.len();

            let read_len = std::cmp::min(buffer_len, out_len);
            for (offset, value) in self.buffer.drain(0..read_len).enumerate() {
                buf[offset] = value;
            }

            Ok(read_len)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::mocks::Receiver as MockReceiver;

    #[tokio::test]
    async fn read_chunk_at_once() {
        let mut tmp_recv = MockReceiver::new();
        tmp_recv.add_chunk(vec![0, 1, 2, 3, 4, 5]);

        let mut recv = Receiver::new(tmp_recv);

        let mut buf = [0; 8];
        let result = recv.read(&mut buf).await;
        assert_eq!(true, result.is_ok());
        assert_eq!(6, result.unwrap());

        assert_eq!(&vec![0, 1, 2, 3, 4, 5], &buf[..6]);
    }

    #[tokio::test]
    async fn read_chunk_in_2_steps() {
        let mut tmp_recv = MockReceiver::new();
        tmp_recv.add_chunk(vec![0, 1, 2, 3, 4, 5]);

        let mut recv = Receiver::new(tmp_recv);

        let mut buf = [0; 3];
        let result = recv.read(&mut buf).await;
        assert_eq!(true, result.is_ok());
        assert_eq!(3, result.unwrap());

        assert_eq!(&vec![0, 1, 2], &buf[..3]);

        let result = recv.read(&mut buf).await;
        assert_eq!(true, result.is_ok());
        assert_eq!(3, result.unwrap());

        assert_eq!(&vec![3, 4, 5], &buf[..3]);
    }

    #[tokio::test]
    async fn read_chunk_in_2_steps_then_another() {
        let mut tmp_recv = MockReceiver::new();
        tmp_recv.add_chunk(vec![0, 1, 2, 3, 4, 5]);
        tmp_recv.add_chunk(vec![6, 7, 8, 9]);

        let mut recv = Receiver::new(tmp_recv);

        let mut buf = [0; 3];
        let result = recv.read(&mut buf).await;
        assert_eq!(true, result.is_ok());
        assert_eq!(3, result.unwrap());
        assert_eq!(&vec![0, 1, 2], &buf[..3]);

        let result = recv.read(&mut buf).await;
        assert_eq!(true, result.is_ok());
        assert_eq!(3, result.unwrap());
        assert_eq!(&vec![3, 4, 5], &buf[..3]);

        let mut buf_2 = [0; 4];
        let result = recv.read(&mut buf_2).await;
        assert_eq!(true, result.is_ok());
        assert_eq!(4, result.unwrap());
        assert_eq!(&vec![6, 7, 8, 9], &buf_2[..4]);
    }
}
