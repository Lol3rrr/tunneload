use std::fmt::{Debug, Formatter};

use async_trait::async_trait;

use general_traits::Sender;

use super::AcceptorMessage;

pub struct AcceptorPluginSender {
    id: i32,
    tx: std::sync::mpsc::Sender<AcceptorMessage>,
}

impl Debug for AcceptorPluginSender {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AcceptorPluginSender ()")
    }
}

impl AcceptorPluginSender {
    pub fn new(id: i32, tx: std::sync::mpsc::Sender<AcceptorMessage>) -> Self {
        Self { id, tx }
    }
}

#[async_trait]
impl Sender for AcceptorPluginSender {
    async fn send(&mut self, data: &[u8]) {
        self.tx
            .send(AcceptorMessage {
                id: self.id,
                data: data.to_vec(),
            })
            .unwrap();
    }
}
