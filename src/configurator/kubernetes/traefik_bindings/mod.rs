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
