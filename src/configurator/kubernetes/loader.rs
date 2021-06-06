use crate::{
    configurator::{
        kubernetes::{ingress, traefik_bindings},
        ActionPluginList, Configurator, MiddlewareList, RuleList, ServiceList,
    },
    internal_services::DashboardEntity,
    tls::{self, auto::CertificateQueue},
};

use async_trait::async_trait;
use futures::{future::join_all, Future, FutureExt};
use kube::Client;
use serde_json::json;
use tokio::join;

use super::traefik_bindings::TraefikParser;

use crate::configurator::kubernetes::events::listen_tls;

/// The actual Loader that loads from the different Kubernetes Ressources
#[derive(Clone)]
pub struct Loader {
    client: Client,
    namespace: String,
    use_traefik: bool,
    use_ingress: bool,
    ingress_priority: u32,
    traefik_parser: TraefikParser,
}

impl Loader {
    /// Creates a new Loader for the given Namespace
    /// with the environment settings used for the Kubernetes-Client
    pub async fn new(namespace: String) -> Self {
        let client = Client::try_default().await.unwrap();

        Self {
            client: client.clone(),
            namespace: namespace.clone(),
            use_traefik: false,
            use_ingress: false,
            ingress_priority: 100,
            traefik_parser: TraefikParser::new(Some(client), Some(namespace)),
        }
    }
    /// Enables the Traefik-CRDs
    pub fn enable_traefik(&mut self) {
        self.use_traefik = true;
    }
    /// Enables the Kubernetes-Ingress Ressources
    pub fn enable_ingress(&mut self) {
        self.use_ingress = true;
    }
    /// The Priority used for Ingress based Configurations
    pub fn set_ingress_priority(&mut self, n_priority: u32) {
        self.ingress_priority = n_priority;
    }
}

#[async_trait]
impl Configurator for Loader {
    fn get_serivce_event_listener(
        &mut self,
        _services: ServiceList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        async fn placeholder() {}

        Box::pin(placeholder().boxed())
    }

    fn get_middleware_event_listener(
        &mut self,
        middlewares: MiddlewareList,
        action_plugins: ActionPluginList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        async fn placeholder() {}

        let traefik_based = if self.use_traefik {
            traefik_bindings::events::listen_middlewares(
                self.client.clone(),
                self.namespace.clone(),
                middlewares,
                action_plugins,
            )
            .boxed()
        } else {
            placeholder().boxed()
        };

        async fn run(futures: std::pin::Pin<Box<dyn Future<Output = ()> + 'static + Send>>) {
            join!(futures);
        }

        Box::pin(run(traefik_based))
    }

    fn get_rules_event_listener(
        &mut self,
        middlewares: MiddlewareList,
        services: ServiceList,
        rules: RuleList,
        cert_queue: Option<CertificateQueue>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        async fn placeholder() {}

        let traefik_based = if self.use_traefik {
            traefik_bindings::events::listen_rules(
                self.client.clone(),
                self.namespace.clone(),
                middlewares,
                services,
                rules.clone(),
                cert_queue,
            )
            .boxed()
        } else {
            placeholder().boxed()
        };

        let ingress_based = if self.use_ingress {
            ingress::events::listen_rules(
                self.client.clone(),
                self.namespace.clone(),
                rules,
                self.ingress_priority,
            )
            .boxed()
        } else {
            placeholder().boxed()
        };

        async fn run(futures: Vec<std::pin::Pin<Box<dyn Future<Output = ()> + 'static + Send>>>) {
            join_all(futures).await;
        }

        Box::pin(run(vec![traefik_based, ingress_based]))
    }

    fn get_tls_event_listener(
        &mut self,
        tls_manager: tls::ConfigManager,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        Box::pin(listen_tls(
            self.client.clone(),
            self.namespace.clone(),
            tls_manager,
        ))
    }
}

/// The Dashboard-Entity for the Kubernetes
/// Configurator
pub struct KubernetesConfigurator {
    traefik: bool,
    ingress: bool,
}

impl KubernetesConfigurator {
    /// Creates a new Empty version of the Entity
    pub fn new() -> Self {
        Self {
            traefik: false,
            ingress: false,
        }
    }
    /// Set the Traefik-Configurator as enabled
    pub fn enable_traefik(&mut self) {
        self.traefik = true;
    }
    /// Set the Ingress-Configurator as enabled
    pub fn enable_ingress(&mut self) {
        self.ingress = true;
    }
}

impl Default for KubernetesConfigurator {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardEntity for KubernetesConfigurator {
    fn get_type(&self) -> &str {
        "Kubernetes"
    }
    fn get_content(&self) -> serde_json::Value {
        json!({
            "traefik": self.traefik,
            "ingress": self.ingress,
        })
    }
}
