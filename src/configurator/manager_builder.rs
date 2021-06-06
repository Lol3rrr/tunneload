use std::sync::Arc;

use crate::{configurator::Configurator, plugins, rules::rule_list::RuleListWriteHandle, tls};

use super::{manager::Manager, parser::GeneralConfigurator};

/// The Builder for creating a single Manager
pub struct ManagerBuilder {
    general_configurators: Vec<Arc<GeneralConfigurator>>,
    configurators: Vec<Box<dyn Configurator + Send>>,
    tls_config: Option<tls::ConfigManager>,
    writer: Option<RuleListWriteHandle>,
    plugin_loader: Option<plugins::Loader>,
}

impl ManagerBuilder {
    /// Creates a new empty Builder
    pub fn new() -> Self {
        Self {
            general_configurators: Vec::new(),
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
        self.configurators.push(Box::new(conf));

        self
    }
    /// Adds the given Configurator to the Configurators List
    pub fn general_configurator(mut self, conf: GeneralConfigurator) -> Self {
        self.general_configurators.push(Arc::new(conf));

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

        Manager::new(
            self.general_configurators,
            self.configurators,
            tls,
            writer,
            self.plugin_loader,
        )
    }
}

impl Default for ManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
