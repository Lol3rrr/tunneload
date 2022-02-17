use std::sync::Arc;

use async_trait::async_trait;
use general::{Group, Name};
use general_traits::Sender;
use rules::{Rule, Service};

use stream_httparse::{Headers, Request, Response, StatusCode};

use super::traits::InternalService;

const SERVICE_NAME: &str = "ready";

/// This Handler is responsible for providing a Readiness Probe to allow external Tools
/// like Kubernetes to determine if an Instance is ready to receive and serve Requests or not
pub struct ReadinessHandler {
    started: std::time::Instant,
}

impl ReadinessHandler {
    /// Creates a new Instance
    pub fn new() -> Self {
        Self {
            started: std::time::Instant::now(),
        }
    }
}

#[async_trait]
impl InternalService for ReadinessHandler {
    #[tracing::instrument(skip(self, sender))]
    async fn handle(
        &self,
        _: &Request<'_>,
        _: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()> {
        let running_duration = self.started.elapsed();

        if running_duration.as_secs() < 60 {
            let response = Response::new(
                "HTTP/1.1",
                StatusCode::ServiceUnavailable,
                Headers::new(),
                Vec::new(),
            );
            sender.send_response(&response).await;

            return Ok(());
        }

        let response = Response::new("HTTP/1.1", StatusCode::OK, Headers::new(), Vec::new());
        sender.send_response(&response).await;

        Ok(())
    }

    fn service(&self) -> Service {
        Service::new(Name::new(SERVICE_NAME, Group::Internal), Vec::new())
    }
    fn check_service(&self, name: &Name) -> bool {
        name.name() == SERVICE_NAME
    }
}
