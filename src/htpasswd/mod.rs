//! Handles all the HTPasswd related stuff, which
//! is mainly needed for the Basic-Auth middleware

mod general;
pub mod md5;
pub use general::{load, Htpasswd};
