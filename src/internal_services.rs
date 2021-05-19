//! This module is responsible for dealing with and managing all
//! the internal Tunneload Services, like the Dashboard

use std::sync::Arc;

use stream_httparse::Request;

use crate::configurator::ConfigItem;
use crate::{acceptors::traits::Sender, rules::Rule};

use self::traits::InternalService;

mod dashboard;
pub use dashboard::{Dashboard, DashboardEntity, DashboardEntityList};

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
}

impl Internals {
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
