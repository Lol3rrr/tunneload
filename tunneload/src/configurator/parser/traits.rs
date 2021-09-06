//! A Collection of Traits needed for the Parser infrastructure of Tunneload

use async_trait::async_trait;
use rules::{Action, Rule, Service};

use std::error::Error;

use super::{
    Event, EventFuture, ParseRuleContext, RawMiddlewareConfig, RawRuleConfig, RawServiceConfig,
    RawTLSConfig, UnimplementedParserError,
};

/// The Generic-Interface that needs to be implemented by a Configurator's-Parser.
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

/// The Generic-Interface that needs to be implementd by Configurator's-Loader.
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

/// The Generic-Interface that needs to bei implemented by a Configurator's-Event-Emitter
#[async_trait]
pub trait EventEmitter: Send + Sync + 'static {
    /// Listens for Service-Events in the Background and sends all the received Events over the
    /// provided Channel
    async fn service_listener(
        &self,
        _sender: tokio::sync::mpsc::UnboundedSender<Event<RawServiceConfig>>,
    ) -> Option<EventFuture> {
        None
    }

    /// Listens for Middleware-Events in the Background and sends all the received Events over the
    /// provided Channel
    async fn middleware_listener(
        &self,
        _sender: tokio::sync::mpsc::UnboundedSender<Event<RawMiddlewareConfig>>,
    ) -> Option<EventFuture> {
        None
    }

    /// Listens for Rule-Events in the Background and sends all the received Events over the
    /// provided Channel
    async fn rule_listener(
        &self,
        _sender: tokio::sync::mpsc::UnboundedSender<Event<RawRuleConfig>>,
    ) -> Option<EventFuture> {
        None
    }

    /// Listens for TLS-Events in the Background and sends all the received Events over the
    /// provided Channel
    async fn tls_listener(
        &self,
        _sender: tokio::sync::mpsc::UnboundedSender<Event<RawTLSConfig>>,
    ) -> Option<EventFuture> {
        None
    }
}
