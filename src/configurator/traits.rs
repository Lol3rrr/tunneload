use crate::tls::{self, auto::CertificateQueue};

use async_trait::async_trait;
use futures::Future;

use super::{ActionPluginList, MiddlewareList, RuleList, ServiceList};

/// This Trait defines the Generic Interface for a
/// Configurator Type
#[async_trait]
pub trait Configurator {
    /// Listens to all the Service related events and updates
    /// the Service configuration based on these events
    fn get_serivce_event_listener(
        &mut self,
        services: ServiceList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

    /// Listens to all the Middleware related events and updates
    /// the Middleware configuration based on these events
    fn get_middleware_event_listener(
        &mut self,
        middlewares: MiddlewareList,
        action_plugins: ActionPluginList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

    /// Listens to all the Rule related events and updates
    /// the Rule configuration based on these events
    fn get_rules_event_listener(
        &mut self,
        middlewares: MiddlewareList,
        services: ServiceList,
        rules: RuleList,
        cert_queue: Option<CertificateQueue>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

    /// Listens to all the TLS-Configuration related events and updates
    /// the TLS-Configuration based on these events
    fn get_tls_event_listener(
        &mut self,
        tls_manager: tls::ConfigManager,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
}
