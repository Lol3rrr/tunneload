//! Handles the way requests "enter" the load-balancer
//! so via Webserver directly or tunneler for example

/// Contains all Client related traits, so to
/// send Data to the Client for example
pub mod traits;

pub mod tunneler;
pub mod webserver;

#[cfg(test)]
pub mod mocks;
