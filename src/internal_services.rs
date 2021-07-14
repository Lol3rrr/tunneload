//! This module is responsible for dealing with and managing all
//! the internal Tunneload Services, like the Dashboard

use std::sync::Arc;

use stream_httparse::Request;

use crate::acceptors::{tunneler, webserver};
use crate::configurator::ConfigItem;
use crate::plugins::PluginAcceptor;
use crate::{acceptors::traits::Sender, rules::Rule};
use crate::{cli, configurator, rules};

use self::traits::InternalService;

mod dashboard;
pub use dashboard::{Dashboard, DashboardEntity, DashboardEntityList};

mod acme;
pub use acme::ChallengeHandler as ACMEHandler;

/// Holds all the Traits needed for handling the internal stuff
pub mod traits;

/// Holds all the information regarding Internal-Services and Handlers,
/// like the Dashboard and all the future services that may be included
pub struct Internals {
    services: Vec<Box<dyn InternalService + Send + Sync>>,
}

impl Internals {
    /// Creates an empty list of internal Services and Handlers
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    /// Registers a new Internal-Service
    pub fn add_service(&mut self, n_value: Box<dyn InternalService + Send + Sync>) {
        self.services.push(n_value);
    }

    /// # Returns
    /// * Ok: The Connection can still be kept open
    /// * Err: The Connection should be closed
    pub async fn handle(
        &self,
        request: &Request<'_>,
        rule: Arc<Rule>,
        sender: &mut dyn Sender,
    ) -> Result<(), ()> {
        let service = rule.service();
        let name = service.name();

        for tmp in self.services.iter() {
            if tmp.check_service(name) {
                return tmp.handle(request, rule, sender).await;
            }
        }

        Err(())
    }
}

impl Default for Internals {
    fn default() -> Self {
        Self {
            services: Vec::new(),
        }
    }
}

impl Internals {
    /// Actually configure and setup all the Internal Services according to the
    /// given Configuration
    pub fn configure(
        &mut self,
        config: &cli::Options,
        config_manager: &mut configurator::Manager,
        read_manager: rules::ReadManager,
        dashboard_configurators: DashboardEntityList,
        plugin_acceptors: &[PluginAcceptor],
    ) {
        // If the internal Dashboard service is enabled, set it up
        if config.dashboard {
            log::info!("Enabled the internal Dashboard");

            let (_, service_list, middleware_list, action_plugin_list) =
                config_manager.get_config_lists();
            let mut internal_dashboard = Dashboard::new(
                read_manager,
                service_list,
                middleware_list,
                DashboardEntityList::new(),
                dashboard_configurators,
                action_plugin_list,
            );

            if let Some(port) = config.webserver.port {
                internal_dashboard.add_acceptor(webserver::WebAcceptor::new(port));
            }
            if let Some(port) = config.webserver.tls_port {
                internal_dashboard.add_acceptor(webserver::WebAcceptor::new(port));
            }
            if config.tunneler.is_normal_enabled() {
                internal_dashboard.add_acceptor(tunneler::TunnelerAcceptor::new(80));
            }
            if config.tunneler.is_tls_enabled() {
                internal_dashboard.add_acceptor(tunneler::TunnelerAcceptor::new(443));
            }

            for plugin_acceptor in plugin_acceptors.iter() {
                internal_dashboard.add_acceptor(plugin_acceptor.dashboard_entity());
            }

            config_manager.register_internal_service(&internal_dashboard);
            self.add_service(Box::new(internal_dashboard));
        }
    }
}
