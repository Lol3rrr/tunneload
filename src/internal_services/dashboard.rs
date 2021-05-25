use std::sync::Arc;

use async_trait::async_trait;
use serde::ser::SerializeSeq;
use serde_json::json;
use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::{
    acceptors::traits::Sender,
    configurator::{MiddlewareList, ServiceList},
    rules::{Matcher, ReadManager, Rule, Service},
};

use super::traits::InternalService;

const SERVICE_NAME: &str = "dashboard@internal";

mod api;
mod file;

/// This holds all the Information needed to provide
/// the Tunneler-Dashboard as an internal Service
pub struct Dashboard {
    rules: ReadManager,
    services: ServiceList,
    middlewares: MiddlewareList,
    acceptors: DashboardEntityList,
    configurators: DashboardEntityList,

    api_matcher: Matcher,
    acceptors_matcher: Matcher,
    configurators_matcher: Matcher,
    rules_matcher: Matcher,
    services_matcher: Matcher,
    middlewares_matcher: Matcher,
}

impl Dashboard {
    /// Creates a new Dashboard
    pub fn new(
        rules: ReadManager,
        services: ServiceList,
        middlewares: MiddlewareList,
        acceptors: DashboardEntityList,
        configurators: DashboardEntityList,
    ) -> Self {
        Self {
            rules,
            services,
            middlewares,
            acceptors,
            configurators,

            api_matcher: Matcher::PathPrefix("/api/".to_owned()),
            acceptors_matcher: Matcher::PathPrefix("/api/acceptors".to_owned()),
            configurators_matcher: Matcher::PathPrefix("/api/configurators".to_owned()),
            rules_matcher: Matcher::PathPrefix("/api/rules".to_owned()),
            services_matcher: Matcher::PathPrefix("/api/services".to_owned()),
            middlewares_matcher: Matcher::PathPrefix("/api/middlewares".to_owned()),
        }
    }

    /// Adds a new Acceptor to the Dashboards config
    pub fn add_acceptor<A>(&mut self, tmp: A)
    where
        A: DashboardEntity + Send + Sync + 'static,
    {
        self.acceptors.push(Box::new(tmp));
    }

    /// Adds a new Configurator to the Dashboards config
    pub fn add_configurator<C>(&mut self, tmp: C)
    where
        C: DashboardEntity + Send + Sync + 'static,
    {
        self.configurators.push(Box::new(tmp));
    }

    async fn handle_api(
        &self,
        request: &Request<'_>,
        _rule: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()> {
        if self.acceptors_matcher.matches(request) {
            return api::handle_acceptors(request, sender, &self.acceptors).await;
        }
        if self.configurators_matcher.matches(request) {
            return api::handle_configurators(request, sender, &self.configurators).await;
        }
        if self.rules_matcher.matches(request) {
            return api::handle_rules(request, sender, &self.rules).await;
        }
        if self.services_matcher.matches(request) {
            return api::handle_services(request, sender, &self.services).await;
        }
        if self.middlewares_matcher.matches(request) {
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
        if self.api_matcher.matches(request) {
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

/// The Bounds needed to register a new Entity on the
/// Dashboard.
/// This will provide all the Information that can then later be accessed
/// and displayed on the Dashboard
pub trait DashboardEntity {
    /// This should uniquely identify the Entity as it otherwise
    /// may lead to confusion when displaying the Data
    ///
    /// This will be included in the form of a "type" Entry in
    /// the generated JSON object
    fn get_type(&self) -> &str;

    /// This should return all the relevant Data for the given
    /// Entity
    ///
    /// This will be included in the form of a "content" Entry in
    /// the generated JSON object
    fn get_content(&self) -> serde_json::Value;
}

/// This simply acts as a Wrapper to make it easier to manage
/// Entities, especially in regards to serializing the Data into
/// JSON or the like
pub struct DashboardEntityList {
    entities: Vec<Box<dyn DashboardEntity + Send + Sync + 'static>>,
}

impl DashboardEntityList {
    /// Creates a new empty List of Entities
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    /// Adds the given Entity to the end of the List
    pub fn push(&mut self, tmp: Box<dyn DashboardEntity + Send + Sync + 'static>) {
        self.entities.push(tmp);
    }
}

impl Default for DashboardEntityList {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
        }
    }
}

impl serde::Serialize for DashboardEntityList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.entities.len()))?;

        for entity in self.entities.iter() {
            let value = json!({
                "type": entity.get_type(),
                "content": entity.get_content(),
            });

            seq.serialize_element(&value)?;
        }

        seq.end()
    }
}
