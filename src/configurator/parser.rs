use std::{error::Error, fmt::Display, sync::Arc};

use crate::{
    rules::{Action, Middleware, Rule, Service},
    tls::{self, auto::CertificateQueue},
};

use async_trait::async_trait;
use futures::Future;
use rustls::sign::CertifiedKey;

use super::{MiddlewareList, PluginList, RuleList, ServiceList};

#[cfg(test)]
pub mod mocks;

#[derive(Debug)]
pub struct ParseRuleContext<'a> {
    pub middlewares: &'a MiddlewareList,
    pub services: &'a ServiceList,
    pub cert_queue: Option<CertificateQueue>,
}

#[derive(Debug)]
pub struct UnimplementedParserError();

impl Display for UnimplementedParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parser is not implemented for this Type")
    }
}

impl Error for UnimplementedParserError {}

#[async_trait]
pub trait Parser: Send + Sync + 'static {
    /// Parses the given Service
    async fn service(&self, _config: &serde_json::Value) -> Result<Service, Box<dyn Error>> {
        Err(Box::new(UnimplementedParserError {}))
    }

    /// Parses the given Action
    ///
    /// # Params:
    /// * `name`: The Name of the Action
    /// * `config`: The Config that belongs to the Action
    async fn parse_action(
        &self,
        _name: &str,
        _config: &serde_json::Value,
    ) -> Result<Action, Box<dyn Error>> {
        Err(Box::new(UnimplementedParserError {}))
    }

    /// Parses the given Rule
    async fn rule<'a>(
        &self,
        _config: &serde_json::Value,
        _context: ParseRuleContext<'a>,
    ) -> Result<Rule, Box<dyn Error>> {
        Err(Box::new(UnimplementedParserError {}))
    }

    ///  Parses the given Config into a useable TLS-Config
    async fn tls(
        &self,
        _config: &serde_json::Value,
    ) -> Result<(String, rustls::sign::CertifiedKey), Box<dyn Error>> {
        Err(Box::new(UnimplementedParserError {}))
    }
}

#[derive(Debug)]
pub struct RawServiceConfig {
    pub config: serde_json::Value,
}

#[derive(Debug)]
pub struct RawMiddlewareConfig {
    pub name: String,
    pub action_name: String,
    pub config: serde_json::Value,
}

#[derive(Debug)]
pub struct RawRuleConfig {
    pub config: serde_json::Value,
}

#[derive(Debug)]
pub struct RawTLSConfig {
    pub config: serde_json::Value,
}

#[async_trait]
pub trait Loader: Send + Sync + 'static {
    /// Loads all the raw serivce configurations which will then be passed
    /// onto the Parser
    async fn services(&self) -> Vec<RawServiceConfig> {
        Vec::new()
    }

    /// Loads all the middlware raw configurations which will then
    /// be passed onto the Parser
    async fn middlewares(&self) -> Vec<RawMiddlewareConfig> {
        Vec::new()
    }

    /// Loads all the raw rule configurations which will then be passed
    /// onto the Parser
    async fn rules(&self) -> Vec<RawRuleConfig> {
        Vec::new()
    }

    /// Loads all the raw tls configurations which will then be passed
    /// onto the Parser
    async fn tls(&self) -> Vec<RawTLSConfig> {
        Vec::new()
    }
}

#[derive(Debug)]
pub enum Event<T> {
    /// This Signals that a ressource has been Updated and contains the
    /// raw Data just like the Loader would return
    Update(T),
    /// Signals that a ressource with the given Name has been removed
    /// and should also be removed from the Configuration accordingly
    Remove(String),
}

pub type EventFuture = std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[async_trait]
pub trait EventEmitter: Send + Sync + 'static {
    async fn service_listener(
        &self,
        _sender: tokio::sync::mpsc::UnboundedSender<Event<RawServiceConfig>>,
    ) -> Option<EventFuture> {
        None
    }

    async fn middleware_listener(
        &self,
        _sender: tokio::sync::mpsc::UnboundedSender<Event<RawMiddlewareConfig>>,
    ) -> Option<EventFuture> {
        None
    }

    async fn rule_listener(
        &self,
        _sender: tokio::sync::mpsc::UnboundedSender<Event<RawRuleConfig>>,
    ) -> Option<EventFuture> {
        None
    }

    async fn tls_listener(
        &self,
        _sender: tokio::sync::mpsc::UnboundedSender<Event<RawTLSConfig>>,
    ) -> Option<EventFuture> {
        None
    }
}

/// The GeneralConfigurator is a general abstraction over a Loader and a Parser
/// to allow for easier control over certain aspects of them as well as
/// allowing for easier reuse of certain parts, like parsers, and better
/// seperation of concerns
pub struct GeneralConfigurator {
    loader: Box<dyn Loader>,
    events: Box<dyn EventEmitter>,
    parser: Box<dyn Parser>,
}

impl GeneralConfigurator {
    /// Creates a new Instance from the given Data
    pub fn new<L, P, E>(loader: L, events: E, parser: P) -> Self
    where
        L: Loader,
        E: EventEmitter,
        P: Parser,
    {
        Self {
            loader: Box::new(loader),
            events: Box::new(events),
            parser: Box::new(parser),
        }
    }

    /// Attempts to load and parse all the Services using the provided Loader and Parser
    pub async fn load_services(&self) -> Vec<Service> {
        let mut result = Vec::new();
        let raw_services = self.loader.services().await;

        for raw_serv in raw_services.iter() {
            match self.parser.service(&raw_serv.config).await {
                Ok(service) => result.push(service),
                Err(e) => {
                    tracing::error!("Parsing Service \n{:?}", e);
                }
            };
        }

        result
    }

    /// Attempts to load and parse the Middlewares using the provided Loader and Parser
    pub async fn load_middlewares(&self, action_plugins: &PluginList) -> Vec<Middleware> {
        let mut result = Vec::new();
        let raw_configs = self.loader.middlewares().await;

        for raw_conf in raw_configs.iter() {
            match parse_middleware(
                &raw_conf.name,
                &raw_conf.action_name,
                &raw_conf.config,
                self.parser.as_ref(),
                action_plugins,
            )
            .await
            {
                Ok(middleware) => {
                    result.push(middleware);
                }
                Err(e) => {
                    tracing::error!("Parsing Middleware: {:?}", e);
                }
            };
        }

        result
    }

    pub async fn load_rules(
        &self,
        middlewares: &MiddlewareList,
        services: &ServiceList,
        cert_queue: Option<CertificateQueue>,
    ) -> Vec<Rule> {
        let mut result = Vec::new();
        let raw_rules = self.loader.rules().await;

        for raw_rule in raw_rules.iter() {
            let context = ParseRuleContext {
                middlewares,
                services,
                cert_queue: cert_queue.clone(),
            };

            match self.parser.rule(&raw_rule.config, context).await {
                Ok(rule) => {
                    result.push(rule);
                }
                Err(e) => {
                    tracing::error!("Parsing Rule: {:?}", e);
                }
            };
        }

        result
    }

    pub async fn load_tls(&self) -> Vec<(String, CertifiedKey)> {
        let mut result = Vec::new();
        let raw_tls = self.loader.tls().await;

        for tmp_tls in raw_tls.iter() {
            match self.parser.tls(&tmp_tls.config).await {
                Ok(tls) => result.push(tls),
                Err(e) => {
                    tracing::error!("Parsing TLS: {:?}", e);
                }
            };
        }

        result
    }

    pub async fn service_events(self: Arc<Self>, services: ServiceList) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let service_future = match self.events.service_listener(tx).await {
            Some(s) => s,
            None => return,
        };

        // Actually run the event emitter returned
        tokio::spawn(service_future);

        loop {
            let event = match rx.recv().await {
                Some(e) => e,
                None => {
                    tracing::error!("Could not receive Event");
                    return;
                }
            };

            match event {
                Event::Update(updated) => {
                    match self.parser.service(&updated.config).await {
                        Ok(updated_service) => {
                            services.set(updated_service);
                        }
                        Err(e) => {
                            tracing::error!("Parsing Service \n{:?}", e);
                        }
                    };
                }
                Event::Remove(name) => {
                    tracing::info!("Removed-Service: {:?}", name);
                }
            };
        }
    }

    pub async fn middleware_events(
        self: Arc<Self>,
        middlewares: MiddlewareList,
        action_plugins: PluginList,
    ) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let middleware_future = match self.events.middleware_listener(tx).await {
            Some(m) => m,
            None => return,
        };

        // Actually run the event emitter returned
        tokio::spawn(middleware_future);

        loop {
            let event = match rx.recv().await {
                Some(e) => e,
                None => {
                    tracing::error!("Could not receive Event");
                    return;
                }
            };

            match event {
                Event::Update(updated) => {
                    match parse_middleware(
                        &updated.name,
                        &updated.action_name,
                        &updated.config,
                        self.parser.as_ref(),
                        &action_plugins,
                    )
                    .await
                    {
                        Ok(middleware) => {
                            middlewares.set(middleware);
                        }
                        Err(e) => {
                            tracing::error!("Parsing Middleware: {:?}", e);
                        }
                    }
                }
                Event::Remove(name) => {
                    tracing::info!("Removed-Middleware: {:?}", name);
                }
            };
        }
    }

    pub async fn rule_events(
        self: Arc<Self>,
        services: ServiceList,
        middlewares: MiddlewareList,
        rules: RuleList,
        cert_queue: Option<CertificateQueue>,
    ) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let rule_future = match self.events.rule_listener(tx).await {
            Some(r) => r,
            None => return,
        };

        // Actually run the event emitter
        tokio::spawn(rule_future);

        loop {
            let event = match rx.recv().await {
                Some(e) => e,
                None => {
                    tracing::error!("Could not receive Event");
                    return;
                }
            };

            match event {
                Event::Update(updated) => {
                    let context = ParseRuleContext {
                        services: &services,
                        middlewares: &middlewares,
                        cert_queue: cert_queue.clone(),
                    };
                    match self.parser.rule(&updated.config, context).await {
                        Ok(rule) => {
                            rules.set_rule(rule);
                        }
                        Err(e) => {
                            tracing::error!("Parsing Rule: {:?}", e);
                        }
                    }
                }
                Event::Remove(name) => {
                    tracing::info!("Removing Rule: {:?}", name);
                    rules.remove_rule(name);
                }
            };
        }
    }

    pub async fn tls_events(self: Arc<Self>, tls_config: tls::ConfigManager) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let tls_future = match self.events.tls_listener(tx).await {
            Some(t) => t,
            None => return,
        };

        // Actually run the event emitter
        tokio::spawn(tls_future);

        loop {
            let event = match rx.recv().await {
                Some(e) => e,
                None => {
                    tracing::error!("Could not receive Event");
                    return;
                }
            };

            match event {
                Event::Update(updated) => {
                    match self.parser.tls(&updated.config).await {
                        Ok(cert) => {
                            tls_config.set_cert(cert);
                        }
                        Err(e) => {
                            tracing::error!("Parsing TLS: {:?}", e);
                        }
                    };
                }
                Event::Remove(name) => {
                    tracing::info!("Removed TLS: {:?}", name);
                }
            };
        }
    }
}

#[derive(Debug)]
pub enum MiddlewareParseError {
    InvalidActionName,
    UnknownPlugin,
    CreatingPluginInstance,
}
impl Display for MiddlewareParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Middleware-Parse-Error")
    }
}
impl Error for MiddlewareParseError {}

/// # Params:
/// * `name`: The Name of the Configured Middleware
/// * `action_name`: The Name of the Middleware/Action to use
/// * `config`: The Configuration to use for the Middleware/Action
pub async fn parse_middleware(
    name: &str,
    action_name: &str,
    config: &serde_json::Value,
    parser: &dyn Parser,
    action_plugins: &PluginList,
) -> Result<Middleware, Box<dyn Error>> {
    let action = if action_name.contains('@') {
        let (name, group) = action_name
            .split_once('@')
            .ok_or_else(|| Box::new(MiddlewareParseError::InvalidActionName))?;

        match group {
            "plugin" => {
                let plugin = action_plugins
                    .get(name)
                    .ok_or_else(|| Box::new(MiddlewareParseError::UnknownPlugin))?;

                let config_str = serde_json::to_string(config).unwrap();
                let instance = plugin
                    .get()
                    .create_instance(config_str)
                    .ok_or_else(|| Box::new(MiddlewareParseError::CreatingPluginInstance))?;

                Action::Plugin(instance)
            }
            _ => return Err(Box::new(MiddlewareParseError::InvalidActionName)),
        }
    } else {
        parser.parse_action(action_name, config).await?
    };

    Ok(Middleware::new(name, action))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::configurator::parser::mocks::MockError;

    use super::mocks::MockParser;
    use super::*;

    #[tokio::test]
    async fn normal_action() {
        assert_eq!(
            Middleware::new("test", Action::Compress),
            parse_middleware(
                "test",
                "compress",
                &json!({}),
                &MockParser::<_, MockError, _, _>::new(
                    Err(MockError {}),
                    Ok(Action::Compress),
                    Err(MockError {}),
                    Err(MockError {}),
                ),
                &PluginList::new()
            )
            .await
            .unwrap()
        );
    }

    #[tokio::test]
    async fn attempt_to_load_plugin() {
        assert_eq!(
            true,
            parse_middleware(
                "test",
                "testplug@plugin",
                &json!({}),
                &MockParser::<_, MockError, _, _>::new(
                    Err(MockError {}),
                    Ok(Action::Compress),
                    Err(MockError {}),
                    Err(MockError {}),
                ),
                &PluginList::new()
            )
            .await
            .is_err()
        );
    }
}
