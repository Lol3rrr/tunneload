/// Ingressroute support for kubernetes traefik
pub mod ingressroute;
/// Middlware support for kubernetes traefik
pub mod middleware;

/// All the event related Stuff for kubernetes-traefik bindings
pub mod events;
///  All the parsing stuff for Traefik-Kubernetes stuff
pub mod parse;

mod load_middlewares;
pub use load_middlewares::load_middlewares;

mod load_endpoints;
pub use load_endpoints::load_endpoints;

mod load_routes;
pub use load_routes::load_routes;
