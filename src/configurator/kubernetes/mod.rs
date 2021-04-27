//! The Kubernetes-Configurator allows users to dynamically configure the
//! Tunneload Instance using different Ressource-Types in a Kubernetes-Cluster

pub mod general_crd;

pub mod events;
pub mod general;
pub mod ingress;
pub mod traefik_bindings;

mod loader;
pub use loader::Loader;
