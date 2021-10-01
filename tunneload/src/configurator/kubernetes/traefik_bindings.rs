//! This loads Configuration from the Traefik Kubernetes CRDs
//!
//! # Rules
//! Loads Rules based on Traefik's HTTP-Route CRDs and basically just loads the entire
//! Configuration just like Traefik would
//!
//! # Middlewares
//! Loads Middlewares based on Traefik's Middleware CRDs and mostly just loads them the same
//! way that Traefik would

/// Ingressroute support for kubernetes traefik
pub mod ingressroute;
/// Middlware support for kubernetes traefik
pub mod middleware;

mod traefik_parser;
pub use traefik_parser::TraefikParser;
mod traefik_loader;
pub use traefik_loader::TraefikLoader;
mod traefik_events;
pub use traefik_events::TraefikEvents;

use crate::configurator::parser::GeneralConfigurator;

/// Creates the General-Configurator for the Traefik-Bindings with the given Namespace and
/// kubernetes client
pub fn setup_general_configurator(client: kube::Client, namespace: &str) -> GeneralConfigurator {
    tracing::info!(
        "Enabling Traefik-Kubernetes-Configurator for namespace: {}",
        namespace
    );

    let traefik_loader = TraefikLoader::new(client.clone(), namespace.to_string());
    let traefik_events = TraefikEvents::new(client.clone(), namespace.to_string());
    let traefik_parser = TraefikParser::new(Some(client), Some("default".to_owned()));

    GeneralConfigurator::new(
        format!("Traefik-{}", namespace),
        traefik_loader,
        traefik_events,
        traefik_parser,
    )
}
