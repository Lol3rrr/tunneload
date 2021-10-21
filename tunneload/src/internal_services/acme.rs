use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use crate::tls::auto::{ChallengeList, ChallengeState};
use general::{Group, Name};
use general_traits::Sender;
use rules::{Rule, Service};

use async_trait::async_trait;
use stream_httparse::{Headers, Request, Response, StatusCode};

use super::traits::InternalService;

const SERVICE_NAME: &str = "acme";

/// The Handler for all ACME and TLS-Certificate Challenges
pub struct ChallengeHandler {
    challenges: ChallengeList,
}

impl ChallengeHandler {
    /// Creates a new Handler with the given ChallengeList
    pub fn new(challenges: ChallengeList) -> Self {
        Self { challenges }
    }
}

impl Debug for ChallengeHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChallengeHandler ()")
    }
}

#[async_trait]
impl InternalService for ChallengeHandler {
    #[tracing::instrument(skip(request, sender))]
    async fn handle(
        &self,
        request: &Request<'_>,
        _: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()> {
        let path = request.path();
        let domain = match request.headers().get("Host") {
            Some(d) => d.to_string(),
            None => {
                tracing::error!("Request did not contain a Host-Header");
                let response = Response::new(
                    "HTTP/1.1",
                    StatusCode::BadRequest,
                    Headers::new(),
                    Vec::new(),
                );
                sender.send_response(&response).await;

                return Ok(());
            }
        };

        let challenge_state = match self.challenges.get_state(&domain) {
            Some(c) => c,
            None => {
                tracing::error!("Could not find a challenge for the Domain: {:?}", domain);
                let response =
                    Response::new("HTTP/1.1", StatusCode::NotFound, Headers::new(), Vec::new());
                sender.send_response(&response).await;

                return Ok(());
            }
        };

        let requested_id = match path.strip_prefix("/.well-known/acme-challenge/") {
            Some(sub) => sub,
            None => {
                tracing::error!(
                    "Requested Path did not match expected Prefix '/.well-known/acme-challenge/': {:?}",
                    path
                );
                let response = Response::new(
                    "HTTP/1.1",
                    StatusCode::BadRequest,
                    Headers::new(),
                    Vec::new(),
                );
                sender.send_response(&response).await;

                return Ok(());
            }
        };

        let challenges = match challenge_state {
            ChallengeState::Data(c) => c,
            _ => {
                tracing::error!(
                    "Configured Challenge is not in the desired State to verify Requests"
                );
                let response = Response::new(
                    "HTTP/1.1",
                    StatusCode::BadRequest,
                    Headers::new(),
                    Vec::new(),
                );
                sender.send_response(&response).await;

                return Ok(());
            }
        };

        let (_, challenge_data) = match challenges.iter().find(|(tmp_id, _)| tmp_id == requested_id)
        {
            Some(c) => c,
            None => {
                tracing::error!("No Challenge matched the provided ID: {:?}", requested_id);
                let response = Response::new(
                    "HTTP/1.1",
                    StatusCode::BadRequest,
                    Headers::new(),
                    Vec::new(),
                );
                sender.send_response(&response).await;

                return Ok(());
            }
        };

        let mut headers = Headers::new();
        headers.set("Content-Length", challenge_data.len());
        let response = Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers,
            challenge_data.as_bytes().to_owned(),
        );
        sender.send_response(&response).await;

        Ok(())
    }

    fn service(&self) -> Service {
        let mut tmp = Service::new(Name::new(SERVICE_NAME, Group::Internal), Vec::new());
        tmp
    }
    fn check_service(&self, name: &Name) -> bool {
        name.name() == SERVICE_NAME
    }
}
