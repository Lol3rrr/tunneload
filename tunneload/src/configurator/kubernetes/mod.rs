//! The Kubernetes-Configurator allows users to dynamically configure the
//! Tunneload Instance using different Ressource-Types in a Kubernetes-Cluster

/// Some general Functionality and Logic related to all CRDs
pub mod general_crd;

pub mod general;
pub mod ingress;
/// The Traefik specific stuff
pub mod traefik_bindings;

mod dashboard;
pub use dashboard::KubernetesConfigurator;

mod setup;
pub use setup::setup;
