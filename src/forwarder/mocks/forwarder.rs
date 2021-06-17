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

pub type MockError = ();

#[async_trait]
impl ForwarderTrait for Forwarder {
    type Connection = ServiceConnection;
    type ConnectError = MockError;

    async fn create_con(&self, _rule: &Rule) -> Result<Self::Connection, Self::ConnectError> {
        Ok(self.con.clone())
    }
}
