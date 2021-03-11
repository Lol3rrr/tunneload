use async_trait::async_trait;

use crate::forwarder::Forwarder as ForwarderTrait;
use crate::{forwarder::mocks::ServiceConnection, rules::Rule};

pub struct Forwarder {
    con: ServiceConnection,
}

impl Forwarder {
    pub fn new(con: ServiceConnection) -> Self {
        Self { con }
    }
}

#[async_trait]
impl ForwarderTrait for Forwarder {
    type Connection = ServiceConnection;

    async fn create_con(&self, _rule: &Rule) -> Option<Self::Connection> {
        Some(self.con.clone())
    }
}
