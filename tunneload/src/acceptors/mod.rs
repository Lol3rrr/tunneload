//! Handles the way requests "enter" the load-balancer
//! so via Webserver directly or tunneler for example

pub mod tunneler;
pub mod webserver;

#[cfg(test)]
pub mod mocks;
