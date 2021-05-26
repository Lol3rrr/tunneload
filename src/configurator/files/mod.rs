//! The File-Configurator allows users to use File(s)
//! to configure the Tunneload Instance

mod loader;
pub use loader::{FileConfigurator, Loader};

mod middlewares;
use middlewares::*;

mod route;
use route::*;

mod config;
pub use config::*;

mod file_parser;
pub use file_parser::FileParser;

/// Creates the Loader + Configurator Pair
pub fn new(path: String) -> (Loader, FileConfigurator) {
    (Loader::new(path.clone()), FileConfigurator::new(path))
}
