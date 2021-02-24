use crate::acceptors::traits::Receiver;

use tunneler_core::message::Message;
use tunneler_core::streams::mpsc::StreamReader;

use async_trait::async_trait;

#[async_trait]
impl Receiver for StreamReader<Message> {
    async fn read(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        match self.recv().await {
            Ok(msg) => {
                if msg.is_eof() {
                    return Ok(0);
                }

                let data = msg.get_data();
                buf.extend_from_slice(data);

                Ok(data.len())
            }
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe)),
        }
    }
}
