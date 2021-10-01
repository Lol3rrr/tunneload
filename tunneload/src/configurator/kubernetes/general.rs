//! This handles all the Configuration using general Kubernetes Configurations
//!
//! # Services
//! A Tunneload-Service is created for each Kubernetes Service with the same Name and
//! Endpoints
//!
//! # TLS
//! Loads TLS Certifcates from Kubernetes tls secrets

mod kubernetes_loader;
pub use kubernetes_loader::KubernetesLoader;
mod kubernetes_parser;
pub use kubernetes_parser::KubernetesParser;
mod kubernetes_events;
pub use kubernetes_events::KubernetesEvents;

use crate::configurator::parser::GeneralConfigurator;

/// Creates the General-Configurator for the Normal Kubernetes configurations with the
/// given Namespace and kubernetes client
pub fn setup_general_configurator(client: kube::Client, namespace: &str) -> GeneralConfigurator {
    tracing::info!(
        "Enabling Kubernetes-Configurator for Namespace: {}",
        namespace
    );

    let k8s_loader = KubernetesLoader::new(client.clone(), namespace.to_string());
    let k8s_events = KubernetesEvents::new(client.clone(), namespace.to_string());
    let k8s_parser = KubernetesParser::new();

    GeneralConfigurator::new(
        format!("K8S-{}", namespace),
        k8s_loader,
        k8s_events,
        k8s_parser,
    )
}
