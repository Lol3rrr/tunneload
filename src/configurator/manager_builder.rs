use crate::configurator::{Configurator, ServiceList};
use crate::rules::WriteManager;
use crate::tls;

use super::manager::Manager;

pub struct ManagerBuilder {
    configurators: Vec<Box<dyn Configurator + Send>>,
    tls_config: Option<tls::ConfigManager>,
    writer: Option<WriteManager>,
    wait_time: Option<std::time::Duration>,
}

impl ManagerBuilder {
    pub fn new() -> Self {
        Self {
            configurators: Vec::new(),
            tls_config: None,
            writer: None,
            wait_time: None,
        }
    }

    pub fn writer(self, writer: WriteManager) -> Self {
        Self {
            configurators: self.configurators,
            tls_config: self.tls_config,
            writer: Some(writer),
            wait_time: self.wait_time,
        }
    }
    pub fn configurator<C: Configurator + Send + 'static>(self, conf: C) -> Self {
        let mut tmp_confs = self.configurators;
        tmp_confs.push(Box::new(conf));

        Self {
            configurators: tmp_confs,
            tls_config: self.tls_config,
            writer: self.writer,
            wait_time: self.wait_time,
        }
    }
    pub fn tls(self, config: tls::ConfigManager) -> Self {
        Self {
            configurators: self.configurators,
            tls_config: Some(config),
            writer: self.writer,
            wait_time: self.wait_time,
        }
    }
    pub fn wait_time(self, config: std::time::Duration) -> Self {
        Self {
            configurators: self.configurators,
            tls_config: self.tls_config,
            writer: self.writer,
            wait_time: Some(config),
        }
    }

    pub fn build(self) -> Manager {
        Manager {
            configurators: self.configurators,
            writer: self.writer.unwrap(),
            tls: self.tls_config.unwrap(),
            services: ServiceList::new(),
            wait_time: self.wait_time.unwrap_or(std::time::Duration::from_secs(30)),
        }
    }
}
