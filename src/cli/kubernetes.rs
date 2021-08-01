use structopt::StructOpt;

/// The Kubernetes specific Options
#[derive(Debug, StructOpt)]
pub struct KubernetesOpts {
    /// The Namespaces to use for general information, like Services and TLS
    #[structopt(long = "kube.namespaces", default_value = "default")]
    pub namespaces: Vec<String>,

    /// Enable the Traefik-Kubernetes-Configurator
    #[structopt(long = "kube.traefik")]
    pub traefik: bool,
    /// The Namespaces to use for loading the Traefik-Configuration
    #[structopt(long = "kube.traefik.namespaces", default_value = "default")]
    pub traefik_namespaces: Vec<String>,

    /// Enable the Ingress-Kubernetes-Configurator
    #[structopt(long = "kube.ingress")]
    pub ingress: bool,
    /// Overwrites the Default priority given to routes
    /// loaded from the Kubernetes-Ingress-Configurator
    #[structopt(long = "kube.ingress.priority")]
    pub ingress_priority: Option<u32>,
    /// The Namespaces to use for loading the Ingress-Routes
    #[structopt(long = "kube.ingress.namespaces", default_value = "default")]
    pub ingress_namespaces: Vec<String>,
}

impl KubernetesOpts {
    /// Checks if either Traefik or Ingress is enabled
    pub fn is_enabled(&self) -> bool {
        self.traefik || self.ingress
    }
}
