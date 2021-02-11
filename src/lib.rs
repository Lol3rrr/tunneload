/// Handles all HTTP-related things, like parsing
/// requests
pub mod http;

/// Handles already parsed requests and then also
/// manages what should be done with a connection
pub mod handler;

/// Handles the way requests "enter" the load-balancer
/// so via Webserver directly or tunneler for example
pub mod acceptors;

/// Handles all the Rule-Matching related stuff
pub mod rules;

/// Manages all the interactions with kubernetes to load
/// the relevant definitions and based on that update
/// the configuration
pub mod kubernetes;

/// Contains some general helper functions that are
/// not specific to one area but might be used in different
/// parts of the project
pub mod general;
