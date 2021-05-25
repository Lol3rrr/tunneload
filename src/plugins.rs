//! This contains all the needed Parts for using WASM-Modules as plugins
//! for different parts of the Load-Balancer to make it more modular
//! and add features without having to work on the general source code

mod middleware;
pub use middleware::MiddlewarePlugin;
