//! The File-Configurator allows users to use File(s)
//! to configure the Tunneload Instance

mod loader;
pub use loader::FileConfigurator;

mod route;
pub use route::{ConfigRoute, ConfigService};

mod config;
pub use config::*;

mod file_parser;
pub use file_parser::FileParser;
mod file_loader;
pub use file_loader::FileLoader;
mod file_events;
pub use file_events::FileEvents;

mod setup;
pub use setup::setup;

/// Creates the Loader + Configurator Pair
pub fn new(path: String) -> FileConfigurator {
    FileConfigurator::new(path)
}
