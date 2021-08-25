use plugins::Plugin;

use lazy_static::lazy_static;
use prometheus::Registry;

use super::ConfigList;

lazy_static! {
    static ref PLUGIN_ACTION_COUNT: prometheus::IntGauge = prometheus::IntGauge::new(
        "config_plugins",
        "The Number of plugins currently registered",
    )
    .unwrap();
}

/// The List of all Plugins currently registered
pub type PluginList = ConfigList<Plugin>;

impl PluginList {
    /// This registers all the Prometheus Metrics related to
    /// service configuration
    pub fn register_metrics(reg: &mut Registry) {
        reg.register(Box::new(PLUGIN_ACTION_COUNT.clone())).unwrap();
    }

    /// Inserts or Updates the given Service in the
    /// List of Services
    pub fn set_plugin_action(&self, n_srv: Plugin) {
        PLUGIN_ACTION_COUNT.set(self.set(n_srv) as i64);
    }
}
