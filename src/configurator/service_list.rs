use crate::{general::Shared, rules::Service};

use lazy_static::lazy_static;
use prometheus::Registry;

lazy_static! {
    static ref CONFIG_SERVICE_COUNT: prometheus::IntGauge = prometheus::IntGauge::new(
        "config_services",
        "The Number of services currently registered",
    )
    .unwrap();
}

#[derive(Debug, Clone)]
pub struct ServiceList(std::sync::Arc<std::sync::Mutex<Vec<Shared<Service>>>>);

impl ServiceList {
    pub fn new() -> Self {
        let services = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        Self(services)
    }

    /// This registers all the Prometheus Metrics related to
    /// service configuration
    pub fn register_metrics(reg: Registry) {
        reg.register(Box::new(CONFIG_SERVICE_COUNT.clone()))
            .unwrap();
    }

    pub fn set_service(&self, n_srv: Service) {
        let mut inner = self.0.lock().unwrap();

        for tmp in inner.iter() {
            let tmp_value = tmp.get();
            if tmp_value.name() == n_srv.name() {
                tmp.update(n_srv);
                return;
            }
        }

        inner.push(Shared::new(n_srv));

        // Updates the Metrics
        CONFIG_SERVICE_COUNT.set(inner.len() as i64);
    }

    pub fn get_service(&self, name: &str) -> Option<Shared<Service>> {
        let inner = self.0.lock().unwrap();

        for tmp in inner.iter() {
            let value = tmp.get();
            if value.name() == name {
                return Some(tmp.clone());
            }
        }

        None
    }
}

impl Default for ServiceList {
    fn default() -> Self {
        Self::new()
    }
}
