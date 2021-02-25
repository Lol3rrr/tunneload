use crate::acceptors::traits::Receiver as ReceiverTrait;

use tunneler_core::message::Message;
use tunneler_core::streams::mpsc::StreamReader;

use async_trait::async_trait;

pub struct Receiver {
    reader: StreamReader<Message>,
    buffer: Vec<u8>,
}

impl Receiver {
    pub fn new(reader: StreamReader<Message>) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl ReceiverTrait for Receiver {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.buffer.is_empty() {
            match self.reader.recv().await {
                Ok(msg) => {
                    if msg.is_eof() {
                        return Ok(0);
                    }

                    let data = msg.get_data();
                    let data_len = data.len();
                    let buf_len = buf.len();

                    let read_len = std::cmp::min(data_len, buf_len);
                    for index in 0..read_len {
                        buf[index] = data[index];
                    }

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
            let mut offset = 0;
            for value in self.buffer.drain(0..read_len) {
                buf[offset] = value;
                offset += 1;
            }

            Ok(read_len)
        }
    }
}
