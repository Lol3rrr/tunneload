//! Handles the all things related to actually
//! forwarding Requests to the Backend-Services

mod traits;
pub use traits::*;

mod basic;
pub use basic::BasicForwarder;

#[cfg(test)]
pub mod mocks;
