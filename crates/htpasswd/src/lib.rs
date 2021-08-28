#![warn(missing_docs)]

//! Handles all the HTPasswd related stuff, which
//! is mainly needed for the Basic-Auth middleware

mod general;
/// All the MD5 related functionality
pub mod md5;
pub use general::{load, Htpasswd};
