use rules::Service;

use lazy_static::lazy_static;
use prometheus::Registry;

use super::{ConfigItem, ConfigList};

lazy_static! {
    static ref CONFIG_SERVICE_COUNT: prometheus::IntGauge = prometheus::IntGauge::new(
        "config_services",
        "The Number of services currently registered",
    )
    .unwrap();
    static ref CONFIG_SERVICE_ENTRIES_COUNT: prometheus::IntGaugeVec =
        prometheus::IntGaugeVec::new(
            prometheus::Opts::new(
                "config_service_entries",
                "The Number of entries for each service",
            ),
            &["service"]
        )
        .unwrap();
}

/// The List of all Services currently registered
pub type ServiceList = ConfigList<Service>;

impl ServiceList {
    /// This registers all the Prometheus Metrics related to
    /// service configuration
    pub fn register_metrics(reg: &mut Registry) {
        reg.register(Box::new(CONFIG_SERVICE_COUNT.clone()))
            .unwrap();
        reg.register(Box::new(CONFIG_SERVICE_ENTRIES_COUNT.clone()))
            .unwrap();
    }

    /// Inserts or Updates the given Service in the
    /// List of Services
    pub fn set_service(&self, n_srv: Service) {
        CONFIG_SERVICE_ENTRIES_COUNT
            .with_label_values(&[n_srv.name()])
            .set(n_srv.address_count() as i64);
        CONFIG_SERVICE_COUNT.set(self.set(n_srv) as i64);
    }
}
