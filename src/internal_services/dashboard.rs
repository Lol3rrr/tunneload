use std::sync::Arc;

use async_trait::async_trait;
use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::{
    acceptors::traits::{Acceptor, Sender},
    configurator::{MiddlewareList, ServiceList},
    rules::{Matcher, ReadManager, Rule, Service},
};

use super::traits::InternalService;

const SERVICE_NAME: &'static str = "dashboard@internal";

mod api;
mod file;

pub struct Dashboard {
    rules: ReadManager,
    services: ServiceList,
    middlewares: MiddlewareList,
    acceptors: Vec<Box<dyn Acceptor + Send + Sync>>,
    configurators: Vec<Box<dyn Configurator + Send + Sync>>,
}

impl Dashboard {
    /// Creates a new Dashboard
    pub fn new(
        rules: ReadManager,
        services: ServiceList,
        middlewares: MiddlewareList,
        acceptors: Vec<Box<dyn Acceptor + Send + Sync>>,
        configurators: Vec<Box<dyn Configurator + Send + Sync>>,
    ) -> Self {
        Self {
            rules,
            services,
            middlewares,
            acceptors,
            configurators,
        }
    }

    pub fn add_acceptor<A>(&mut self, tmp: A)
    where
        A: Acceptor + Send + Sync + 'static,
    {
        self.acceptors.push(Box::new(tmp));
    }
    pub fn add_configurator<C>(&mut self, tmp: C)
    where
        C: Configurator + Send + Sync + 'static,
    {
        self.configurators.push(Box::new(tmp));
    }

    async fn handle_api(
        &self,
        request: &Request<'_>,
        rule: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()> {
        log::info!("Handling API");

        let acceptors_matcher = Matcher::PathPrefix("/api/acceptors".to_owned());
        if acceptors_matcher.matches(request) {
            return api::handle_acceptors(request, sender, &self.acceptors).await;
        }

        let configurators_matcher = Matcher::PathPrefix("/api/configurators".to_owned());
        if configurators_matcher.matches(request) {
            return api::handle_configurators(request, sender, &self.configurators).await;
        }

        let rules_matcher = Matcher::PathPrefix("/api/rules".to_owned());
        if rules_matcher.matches(request) {
            return api::handle_rules(request, sender, &self.rules).await;
        }

        let services_matcher = Matcher::PathPrefix("/api/services".to_owned());
        if services_matcher.matches(request) {
            return api::handle_services(request, sender, &self.services).await;
        }

        let middlewares_matcher = Matcher::PathPrefix("/api/middlewares".to_owned());
        if middlewares_matcher.matches(request) {
            return api::handle_middlewares(request, sender, &self.middlewares).await;
        }

        let mut headers = Headers::new();
        headers.append("Content-Length", 0);
        let response = Response::new("HTTP/1.1", StatusCode::NotFound, headers, vec![]);

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

        file::handle_file(request, rule, sender).await
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

pub trait Configurator {
    fn serialize(&self) -> serde_json::Value;
}
