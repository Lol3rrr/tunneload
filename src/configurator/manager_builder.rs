use crate::configurator::Configurator;
use crate::rules::WriteManager;
use crate::tls;

use super::manager::Manager;

pub struct ManagerBuilder {
    configurators: Vec<Box<dyn Configurator + Send>>,
    tls_config: Option<tls::ConfigManager>,
    writer: Option<WriteManager>,
}

impl ManagerBuilder {
    pub fn new() -> Self {
        Self {
            configurators: Vec::new(),
            tls_config: None,
            writer: None,
        }
    }

    pub fn writer(self, writer: WriteManager) -> Self {
        Self {
            configurators: self.configurators,
            tls_config: self.tls_config,
            writer: Some(writer),
        }
    }
    pub fn configurator<C: Configurator + Send + 'static>(self, conf: C) -> Self {
        let mut tmp_confs = self.configurators;
        tmp_confs.push(Box::new(conf));

        Self {
            configurators: tmp_confs,
            tls_config: self.tls_config,
            writer: self.writer,
        }
    }
    pub fn tls(self, config: tls::ConfigManager) -> Self {
        Self {
            configurators: self.configurators,
            tls_config: Some(config),
            writer: self.writer,
        }
    }

    pub fn build(self) -> Manager {
        Manager {
            configurators: self.configurators,
            writer: self.writer.unwrap(),
            tls: self.tls_config.unwrap(),
            services: Vec::new(),
        }
    }
}
