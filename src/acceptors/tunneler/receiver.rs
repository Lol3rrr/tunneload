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
