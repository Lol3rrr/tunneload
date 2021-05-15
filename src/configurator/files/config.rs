use serde::Deserialize;

use crate::configurator::files::{ConfigMiddleware, ConfigRoute};

/// The underlying File Structure
#[derive(Debug, Deserialize)]
pub struct Config {
    /// The List of Middlewares that can be defined in a
    /// Config file
    pub middleware: Option<Vec<ConfigMiddleware>>,
    /// The List of Routes defined in a Config File
    pub routes: Option<Vec<ConfigRoute>>,
}
