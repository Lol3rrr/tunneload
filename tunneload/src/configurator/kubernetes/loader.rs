use crate::internal_services::DashboardEntity;

use serde_json::json;

/// The Dashboard-Entity for the Kubernetes
/// Configurator
pub struct KubernetesConfigurator {
    traefik: bool,
    ingress: bool,
}

impl KubernetesConfigurator {
    /// Creates a new Empty version of the Entity
    pub fn new() -> Self {
        Self {
            traefik: false,
            ingress: false,
        }
    }
    /// Set the Traefik-Configurator as enabled
    pub fn enable_traefik(&mut self) {
        self.traefik = true;
    }
    /// Set the Ingress-Configurator as enabled
    pub fn enable_ingress(&mut self) {
        self.ingress = true;
    }
}

impl Default for KubernetesConfigurator {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardEntity for KubernetesConfigurator {
    fn get_type(&self) -> &str {
        "Kubernetes"
    }
    fn get_content(&self) -> serde_json::Value {
        json!({
            "traefik": self.traefik,
            "ingress": self.ingress,
        })
    }
}
