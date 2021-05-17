use std::sync::Arc;

use async_trait::async_trait;
use stream_httparse::Request;

use crate::{acceptors::traits::Sender, rules::Rule};

#[async_trait]
pub trait InternalService {
    async fn handle(
        &self,
        request: &Request<'_>,
        rule: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()>;

    fn check_service(&self, name: &str) -> bool;
}
