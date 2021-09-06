use std::{
    error::Error,
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::tls::{self, auto::CertificateQueue};
use rules::{Action, Middleware, Rule, Service};

use futures::Future;
use rustls::sign::CertifiedKey;

use super::{MiddlewareList, PluginList, RuleList, ServiceList};

#[cfg(test)]
pub mod mocks;

pub mod traits;
pub use traits::*;

/// The Context containing all the Data that could be needed for parsing a new
/// Rule
#[derive(Debug)]
pub struct ParseRuleContext<'a> {
    /// The List of Middlewares
    pub middlewares: &'a MiddlewareList,
    /// The List of Services
    pub services: &'a ServiceList,
    /// A Queue for Certificates to generate
    pub cert_queue: Option<CertificateQueue>,
}

/// The Error returned by all the Default implementations for the Parser-Trait
/// to indicate that the Parser does not implement the given Function
#[derive(Debug)]
pub struct UnimplementedParserError();

impl Display for UnimplementedParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parser is not implemented for this Type")
    }
}

impl Error for UnimplementedParserError {}

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

/// The GeneralConfigurator is a general abstraction over a Loader and a Parser
/// to allow for easier control over certain aspects of them as well as
/// allowing for easier reuse of certain parts, like parsers, and better
/// seperation of concerns
pub struct GeneralConfigurator {
    name: String,
    loader: Box<dyn Loader>,
    events: Box<dyn EventEmitter>,
    parser: Box<dyn Parser>,
}

impl Debug for GeneralConfigurator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GeneralConfigurator ( name = {} )", self.name)
    }
}

impl GeneralConfigurator {
    /// Creates a new Instance from the given Data
    pub fn new<L, P, E>(name: String, loader: L, events: E, parser: P) -> Self
    where
        L: Loader,
        E: EventEmitter,
        P: Parser,
    {
        Self {
            name,
            loader: Box::new(loader),
            events: Box::new(events),
            parser: Box::new(parser),
        }
    }

    /// Attempts to load and parse all the Services using the provided Loader and Parser
    #[tracing::instrument]
    pub async fn load_services(&self) -> Vec<Service> {
        let mut result = Vec::new();
        let raw_services = self.loader.services().await;

        tracing::debug!("Raw-Service-Count: {}", raw_services.len());

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
    #[tracing::instrument(skip(action_plugins))]
    pub async fn load_middlewares(&self, action_plugins: &PluginList) -> Vec<Middleware> {
        let mut result = Vec::new();
        let raw_configs = self.loader.middlewares().await;

        tracing::debug!("Raw-Middleware-Count: {}", raw_configs.len());

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

    /// Attempts to load and parse the Rules using the provided Loader and Parser
    ///
    /// # Params
    /// * `middlewares`: All the currently registered Middlewares
    /// * `services`: All the currently registered Services
    /// * `cert_queue`: The Queue to request Certificates for certain Domains, if set this will also
    /// mark the Rules-TLS as Generate
    #[tracing::instrument(skip(middlewares, services, cert_queue))]
    pub async fn load_rules(
        &self,
        middlewares: &MiddlewareList,
        services: &ServiceList,
        cert_queue: Option<CertificateQueue>,
    ) -> Vec<Rule> {
        let mut result = Vec::new();
        let raw_rules = self.loader.rules().await;

        tracing::debug!("Raw-Rules-Count: {}", raw_rules.len());

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

    #[tracing::instrument]
    pub async fn load_tls(&self) -> Vec<(String, CertifiedKey)> {
        let mut result = Vec::new();
        let raw_tls = self.loader.tls().await;

        tracing::debug!("Raw-TLS-Count: {}", raw_tls.len());

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
