pub mod general_crd;

pub mod events;
pub mod general;
pub mod ingress;
pub mod traefik_bindings;

mod loader;
pub use loader::Loader;
