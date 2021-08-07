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
