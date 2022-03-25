use std::sync::Arc;

use async_trait::async_trait;
use general::{Group, Name};
use general_traits::Sender;
use rules::{Rule, Service};

use stream_httparse::{Headers, Request, Response, StatusCode};

use super::traits::InternalService;

const SERVICE_NAME: &str = "status";

/// This Handler is responsible for providing a Readiness and Liveness Probe to allow external
/// Tools like Kubernetes to determine if an Instance is ready to receive and serve Requests
/// or not
pub struct StatusHandler {
    started: std::time::Instant,
}

impl StatusHandler {
    /// Creates a new Instance
    pub fn new() -> Self {
        Self {
            started: std::time::Instant::now(),
        }
    }

    async fn handle_ready(&self, sender: &mut dyn Sender) -> Result<(), ()> {
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

    async fn handle_live(&self, sender: &mut dyn Sender) -> Result<(), ()> {
        let response = Response::new("HTTP/1.1", StatusCode::OK, Headers::new(), Vec::new());
        sender.send_response(&response).await;

        Ok(())
    }
}

impl Default for StatusHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InternalService for StatusHandler {
    #[tracing::instrument(skip(self, sender))]
    async fn handle(
        &self,
        req: &Request<'_>,
        _: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()> {
        let req_path: &str = req.path();

        if req_path.starts_with("/ready") {
            return self.handle_ready(sender).await;
        }

        if req_path.starts_with("/live") {
            return self.handle_live(sender).await;
        }

        tracing::error!("Unexpected Request to Status: {:?}", req);

        Ok(())
    }

    fn service(&self) -> Service {
        Service::new(Name::new(SERVICE_NAME, Group::Internal), Vec::new())
    }
    fn check_service(&self, name: &Name) -> bool {
        name.name() == SERVICE_NAME
    }
}
