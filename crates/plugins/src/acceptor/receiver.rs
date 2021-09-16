use std::fmt::{Debug, Formatter};

use async_trait::async_trait;

use general_traits::Receiver;

pub struct AcceptorPluginReceiver {
    buffered: Option<Vec<u8>>,
    rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
}

impl Debug for AcceptorPluginReceiver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AcceptorPluginReceiver ()")
    }
}

impl AcceptorPluginReceiver {
    pub fn new(rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>) -> Self {
        Self { buffered: None, rx }
    }
}

#[async_trait]
impl Receiver for AcceptorPluginReceiver {
    async fn read(&mut self, target: &mut [u8]) -> std::io::Result<usize> {
        if let Some(tmp) = self.buffered.as_mut() {
            if target.len() >= tmp.len() {
                let length = tmp.len();

                target[0..length].copy_from_slice(tmp);
                self.buffered = None;
                return Ok(length);
            } else {
                let length = target.len();

                target.copy_from_slice(&tmp[..length]);
                tmp.copy_within(length.., 0);
                tmp.truncate(tmp.len() - length);
                return Ok(length);
            }
        }

        let mut data = match self.rx.recv().await {
            Some(d) => d,
            None => return Ok(0),
        };

        if target.len() >= data.len() {
            let length = data.len();
            target[..length].copy_from_slice(&data);
            Ok(length)
        } else {
            let length = target.len();
            target.copy_from_slice(&data[..length]);

            data.copy_within(length.., 0);
            data.truncate(data.len() - length);
            self.buffered = Some(data);

            Ok(length)
        }
    }
}
