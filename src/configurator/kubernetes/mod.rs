pub mod general_crd;

pub mod ingress;
pub mod traefik_bindings;

mod loader;
pub use loader::Loader;
