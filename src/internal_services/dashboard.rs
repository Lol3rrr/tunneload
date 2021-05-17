use std::sync::Arc;

use async_trait::async_trait;
use rust_embed::RustEmbed;
use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::{
    acceptors::traits::Sender,
    rules::{Matcher, Rule, Service},
};

use super::traits::InternalService;

const SERVICE_NAME: &'static str = "dashboard@internal";

#[derive(RustEmbed)]
#[folder = "src/internal_services/dashboard/website/public/"]
struct WebsiteFolder;

pub struct Dashboard {}

impl Dashboard {
    /// Creates a new Dashboard
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_file(
        &self,
        request: &Request<'_>,
        rule: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()> {
        let raw_path = request.path().trim_start_matches('/');
        let raw_path = if raw_path.chars().last() == Some('/') || raw_path.len() == 0 {
            format!("{}index.html", raw_path)
        } else {
            raw_path.to_owned()
        };

        let (path, content_type) = match raw_path.rsplit_once('.') {
            Some((_, ending)) => {
                let c_type = match ending {
                    "html" => "text/html",
                    "js" => "text/javascript",
                    "css" => "text/css",
                    _ => "text",
                };

                (raw_path.to_owned(), c_type)
            }
            None => (format!("{}.html", raw_path), "text/html"),
        };

        let file = match WebsiteFolder::get(&path) {
            Some(content) => content,
            None => {
                log::error!("Could not load File");
                return Err(());
            }
        };

        let mut headers = Headers::new();
        headers.append("Content-Type", content_type);
        let content = match file {
            std::borrow::Cow::Borrowed(val) => {
                headers.append("Content-Length", val.len());
                val.to_vec()
            }
            std::borrow::Cow::Owned(val) => {
                headers.append("Content-Length", val.len());
                val.to_vec()
            }
        };
        let response = Response::new("HTTP/1.1", StatusCode::OK, headers, content);

        let (response_head, response_body) = response.serialize();
        let head_length = response_head.len();
        sender.send(response_head, head_length).await;
        let body_length = response_body.len();
        sender.send(response_body.to_vec(), body_length).await;

        Ok(())
    }

    async fn handle_api(
        &self,
        request: &Request<'_>,
        rule: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()> {
        log::info!("Handling API");

        let mut headers = Headers::new();
        headers.append("Content-Length", 0);
        let response = Response::new("HTTP/1.1", StatusCode::OK, headers, vec![]);

        let (response_head, response_body) = response.serialize();
        let head_length = response_head.len();
        sender.send(response_head, head_length).await;
        let body_length = response_body.len();
        sender.send(response_body.to_vec(), body_length).await;

        Ok(())
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
        let api_matcher = Matcher::PathPrefix("/api/".to_owned());
        if api_matcher.matches(request) {
            return self.handle_api(request, rule, sender).await;
        }

        self.handle_file(request, rule, sender).await
    }

    fn check_service(&self, name: &str) -> bool {
        name == SERVICE_NAME
    }

    fn service(&self) -> Service {
        let mut tmp = Service::new(SERVICE_NAME, Vec::new());
        tmp.set_internal(true);
        tmp
    }
}
