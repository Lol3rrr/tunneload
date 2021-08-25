//! All the Websocket related functionality

pub mod handshake;
mod is_websocket;
pub use is_websocket::is_websocket;

mod dataframe;
pub use dataframe::DataFrame;
