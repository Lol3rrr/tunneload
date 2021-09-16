#![warn(missing_docs)]
//! Contains some general helper functions that are
//! not specific to one area but might be used in different
//! parts of the project

mod parse_time;
pub use parse_time::parse_time;

mod shared;
pub use shared::Shared;
