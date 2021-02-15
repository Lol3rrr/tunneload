use serde::Deserialize;

use crate::configurator::files::{ConfigMiddleware, ConfigRoute};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub middleware: Option<Vec<ConfigMiddleware>>,
    pub routes: Option<Vec<ConfigRoute>>,
}
