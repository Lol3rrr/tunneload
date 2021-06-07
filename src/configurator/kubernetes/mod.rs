//! The Kubernetes-Configurator allows users to dynamically configure the
//! Tunneload Instance using different Ressource-Types in a Kubernetes-Cluster

/// Some general Functionality and Logic related to all CRDs
pub mod general_crd;

/// The general Kubernetes-Configuration stuff, like Services and TLS
pub mod general;
/// The Ingress specific stuff
pub mod ingress;
/// The Traefik specific stuff
pub mod traefik_bindings;

mod loader;
pub use loader::KubernetesConfigurator;

mod setup;
pub use setup::setup;
