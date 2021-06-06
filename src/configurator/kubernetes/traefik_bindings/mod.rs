/// Ingressroute support for kubernetes traefik
pub mod ingressroute;
/// Middlware support for kubernetes traefik
pub mod middleware;

/// All the event related Stuff for kubernetes-traefik bindings
pub mod events;
///  All the parsing stuff for Traefik-Kubernetes stuff
pub mod parse;

mod traefik_parser;
pub use traefik_parser::TraefikParser;
mod traefik_loader;
pub use traefik_loader::TraefikLoader;
mod traefik_events;
pub use traefik_events::TraefikEvents;
