use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct KubernetesOpts {
    /// Enable the Traefik-Kubernetes-Configurator
    #[structopt(long = "kube.traefik")]
    pub traefik: bool,

    /// Enable the Ingress-Kubernetes-Configurator
    #[structopt(long = "kube.ingress")]
    pub ingress: bool,
    /// Overwrites the Default priority given to routes
    /// loaded from the Kubernetes-Ingress-Configurator
    #[structopt(long = "kube.ingress.priority")]
    pub ingress_priority: Option<u32>,
}

impl KubernetesOpts {
    pub fn is_enabled(&self) -> bool {
        self.traefik || self.ingress
    }
}
