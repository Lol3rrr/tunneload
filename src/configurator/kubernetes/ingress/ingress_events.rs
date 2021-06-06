use async_trait::async_trait;

use crate::configurator::parser::EventEmitter;

pub struct IngressEvents {
    client: kube::Client,
    namespace: String,
}

impl IngressEvents {
    pub fn new(client: kube::Client, namespace: String) -> Self {
        Self { client, namespace }
    }
}

#[async_trait]
impl EventEmitter for IngressEvents {}
