//! Handles already parsed requests and then also
//! manages what should be done with a connection

pub mod traits;

mod basic;
pub use basic::BasicHandler;
