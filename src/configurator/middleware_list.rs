use crate::rules::Middleware;

use lazy_static::lazy_static;
use prometheus::Registry;

use super::ConfigList;

lazy_static! {
    static ref CONFIG_MIDDLEWARE_COUNT: prometheus::IntGauge = prometheus::IntGauge::new(
        "config_middlewares",
        "The Number of middlewares currently registered",
    )
    .unwrap();
}

pub type MiddlewareList = ConfigList<Middleware>;

impl MiddlewareList {
    /// This registers all the Prometheus Metrics related to
    /// service configuration
    pub fn register_metrics(reg: &mut Registry) {
        reg.register(Box::new(CONFIG_MIDDLEWARE_COUNT.clone()))
            .unwrap();
    }

    pub fn set_middleware(&self, n_mid: Middleware) {
        CONFIG_MIDDLEWARE_COUNT.set(self.set(n_mid) as i64);
    }

    pub fn remove_middleware<S>(&self, name: S)
    where
        S: AsRef<str>,
    {
        CONFIG_MIDDLEWARE_COUNT.set(self.remove(name) as i64);
    }
}
