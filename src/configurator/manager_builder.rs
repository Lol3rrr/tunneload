use crate::plugins;
use crate::tls;
use crate::{configurator::Configurator, rules::rule_list::RuleListWriteHandle};

use super::manager::Manager;

/// The Builder for creating a single Manager
pub struct ManagerBuilder {
    configurators: Vec<Box<dyn Configurator + Send>>,
    tls_config: Option<tls::ConfigManager>,
    writer: Option<RuleListWriteHandle>,
    plugin_loader: Option<plugins::Loader>,
}

impl ManagerBuilder {
    /// Creates a new empty Builder
    pub fn new() -> Self {
        Self {
            configurators: Vec::new(),
            tls_config: None,
            writer: None,
            plugin_loader: None,
        }
    }

    /// Sets the Writer that should be used
    pub fn writer(mut self, writer: RuleListWriteHandle) -> Self {
        self.writer = Some(writer);

        self
    }
    /// Adds the given Configurator to the Configurators List
    pub fn configurator<C: Configurator + Send + 'static>(mut self, conf: C) -> Self {
        let mut tmp_confs = self.configurators;
        tmp_confs.push(Box::new(conf));

        self.configurators = tmp_confs;

        self
    }
    /// Sets the TLS-ConfigManager
    pub fn tls(mut self, config: tls::ConfigManager) -> Self {
        self.tls_config = Some(config);

        self
    }

    /// Sets the Plugin-Loader
    pub fn plugin_loader(mut self, loader: plugins::Loader) -> Self {
        self.plugin_loader = Some(loader);

        self
    }

    /// Builds the final Manager from the configured
    /// Settings in the Builder
    pub fn build(self) -> Manager {
        let tls = self.tls_config.unwrap();
        let writer = self.writer.unwrap();
        let plugin_loader = self.plugin_loader.unwrap();

        Manager::new(self.configurators, tls, writer, plugin_loader)
    }
}

impl Default for ManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
