/// Handles all HTTP-related things, like parsing
/// requests
pub mod http;

/// Handles already parsed requests and then also
/// manages what should be done with a connection
pub mod handler;

/// Handles the way requests "enter" the load-balancer
/// so via Webserver directly or tunneler for example
pub mod acceptors;
