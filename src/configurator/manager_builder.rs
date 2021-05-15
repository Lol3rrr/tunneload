use crate::tls;
use crate::{
    configurator::{Configurator, MiddlewareList, ServiceList},
    rules::rule_list::RuleListWriteHandle,
};

use super::{manager::Manager, RuleList};

/// The Builder for creating a single Manager
pub struct ManagerBuilder {
    configurators: Vec<Box<dyn Configurator + Send>>,
    tls_config: Option<tls::ConfigManager>,
    writer: Option<RuleListWriteHandle>,
}

impl ManagerBuilder {
    /// Creates a new empty Builder
    pub fn new() -> Self {
        Self {
            configurators: Vec::new(),
            tls_config: None,
            writer: None,
        }
    }

    /// Sets the Writer that should be used
    pub fn writer(self, writer: RuleListWriteHandle) -> Self {
        Self {
            configurators: self.configurators,
            tls_config: self.tls_config,
            writer: Some(writer),
        }
    }
    /// Adds the given Configurator to the Configurators List
    pub fn configurator<C: Configurator + Send + 'static>(self, conf: C) -> Self {
        let mut tmp_confs = self.configurators;
        tmp_confs.push(Box::new(conf));

        Self {
            configurators: tmp_confs,
            tls_config: self.tls_config,
            writer: self.writer,
        }
    }
    /// Sets the TLS-ConfigManager
    pub fn tls(self, config: tls::ConfigManager) -> Self {
        Self {
            configurators: self.configurators,
            tls_config: Some(config),
            writer: self.writer,
        }
    }

    /// Builds the final Manager from the configured
    /// Settings in the Builder
    pub fn build(self) -> Manager {
        Manager {
            configurators: self.configurators,
            tls: self.tls_config.unwrap(),
            services: ServiceList::new(),
            middlewares: MiddlewareList::new(),
            rules: RuleList::new(self.writer.unwrap()),
        }
    }
}

impl Default for ManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
