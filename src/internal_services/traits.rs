use std::sync::Arc;

use async_trait::async_trait;
use stream_httparse::Request;

use crate::{
    acceptors::traits::Sender,
    rules::{Rule, Service},
};

/// This defines the Bounds for a single Internal Service
#[async_trait]
pub trait InternalService {
    /// If a Request is bound for this Service, this function
    /// will be called with all the needed information to
    /// correctly handle this single Request
    async fn handle(
        &self,
        request: &Request<'_>,
        rule: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()>;

    /// Checks if the given Name matches up with the service's
    /// internal Name, example "dashboard@internal"
    fn check_service(&self, name: &str) -> bool;

    /// The Service configuration that can then be targeted
    /// by Rules to access the Service
    fn service(&self) -> Service;
}
