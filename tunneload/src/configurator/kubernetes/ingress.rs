//! This is responsible for loading and parsing the Configurator using Kubernetes
//! Ingress.
//!
//! # Rules
//! Rules are created and updated based on Ingress Rules
//! ## Rule-Matchers
//! The Matchers for a Rule are simply obtained from the Ingress-HTTP spec which provides a Path,
//! which will be used for a PathPrefix-Matcher, and a Domain, which will be used for a
//! Domain-Matcher.
//! ## Rule-Service
//! The Service is simply obtained from the Target service of the Ingress-Rule and then searched
//! for in Tunneloads internal list of Services
//! ## Rule-Middlewares
//! Middlewares for a Ingress-Rule are set using a custom annotation on the Ingress Object. To set
//! a Middleware you simply need to add an Annotation with the `tunneload-middleware` Key and a
//! comma seperated List of the Names of the Middlewares you want to use as the Value.

mod ingress_loader;
pub use ingress_loader::IngressLoader;
mod ingress_parser;
pub use ingress_parser::IngressParser;
mod ingress_events;
pub use ingress_events::IngressEvents;

use crate::configurator::parser::GeneralConfigurator;

/// Creates the General-Configurator for the Ingress configurations with the given
/// Namespace, kubernetes client and defalt priority
pub fn setup_general_configurator(
    client: kube::Client,
    namespace: &str,
    priority: u32,
) -> GeneralConfigurator {
    tracing::info!(
        "Enabling Ingress-Kubernetes-Configurator for namespace: {}",
        namespace
    );

    let ingress_loader = IngressLoader::new(client.clone(), namespace.to_string());
    let ingress_events = IngressEvents::new(client.clone(), namespace.to_string());
    let ingress_parser = IngressParser::new(priority);

    GeneralConfigurator::new(
        format!("Ingress-{}", namespace),
        ingress_loader,
        ingress_events,
        ingress_parser,
    )
}
