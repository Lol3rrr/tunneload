use argser::argser;

/// The Kubernetes specific Options
#[argser]
#[derive(Debug)]
pub struct KubernetesOpts {
    /// The Namespaces to use for general information, like Services and TLS
    #[argser(default_func(default_namespaces))]
    pub namespaces: Vec<String>,

    /// Enable the Traefik-Kubernetes-Configurator
    #[argser(default)]
    pub traefik: bool,
    /// The Namespaces to use for loading the Traefik-Configuration
    #[argser(default_func(default_namespaces))]
    pub traefik_namespaces: Vec<String>,

    /// Enable the Ingress-Kubernetes-Configurator
    #[argser(default)]
    pub ingress: bool,
    /// Overwrites the Default priority given to routes
    /// loaded from the Kubernetes-Ingress-Configurator
    #[argser(default)]
    pub ingress_priority: Option<u32>,
    /// The Namespaces to use for loading the Ingress-Routes
    #[argser(default_func(default_namespaces))]
    pub ingress_namespaces: Vec<String>,
}

fn default_namespaces() -> Vec<String> {
    vec!["default".to_string()]
}

impl KubernetesOpts {
    /// Checks if either Traefik or Ingress is enabled
    pub fn is_enabled(&self) -> bool {
        self.traefik || self.ingress
    }
}
