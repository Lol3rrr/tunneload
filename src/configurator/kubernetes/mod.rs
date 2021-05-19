//! The Kubernetes-Configurator allows users to dynamically configure the
//! Tunneload Instance using different Ressource-Types in a Kubernetes-Cluster

/// Some general Functionality and Logic related to all CRDs
pub mod general_crd;

/// General Event related parts
pub mod events;
/// Some general functionality that is used by different Kubernetes parts
pub mod general;
/// The Ingress specific stuff
pub mod ingress;
/// The Traefik specific stuff
pub mod traefik_bindings;

mod loader;
pub use loader::{KubernetesConfigurator, Loader};
