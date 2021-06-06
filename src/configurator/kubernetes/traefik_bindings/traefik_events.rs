use async_trait::async_trait;

use crate::configurator::parser::{Event, EventEmitter, RawServiceConfig};

pub struct TraefikEvents {
    client: kube::Client,
    namespace: String,
}

impl TraefikEvents {
    pub fn new(client: kube::Client, namespace: String) -> Self {
        Self { client, namespace }
    }
}

#[async_trait]
impl EventEmitter for TraefikEvents {}
