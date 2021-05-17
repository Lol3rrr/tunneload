use std::sync::Arc;

use async_trait::async_trait;
use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::{
    acceptors::traits::Sender,
    rules::{Rule, Service},
};

use super::traits::InternalService;

const SERVICE_NAME: &'static str = "dashboard@internal";

pub struct Dashboard {}

impl Dashboard {
    /// Creates a new Dashboard
    pub fn new() -> Self {
        Self {}
    }

    /// The Service under which the Dashboard can be
    /// accessed
    pub fn service() -> Service {
        let mut tmp = Service::new(SERVICE_NAME, Vec::new());
        tmp.set_internal(true);
        tmp
    }
}

#[async_trait]
impl InternalService for Dashboard {
    async fn handle(
        &self,
        request: &Request<'_>,
        rule: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()> {
        log::info!("Received Dashboard Request");
        log::info!("Request: {:?}", request);

        let response = Response::new("HTTP/1.1", StatusCode::OK, Headers::new(), vec![]);

        let (response_head, response_body) = response.serialize();
        let head_length = response_head.len();
        sender.send(response_head, head_length).await;
        let body_length = response_body.len();
        sender.send(response_body.to_vec(), body_length).await;

        Ok(())
    }

    fn check_service(&self, name: &str) -> bool {
        name == SERVICE_NAME
    }
}
