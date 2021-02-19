use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    /// Enable the Traefik-Kubernetes-Configurator
    #[structopt(long = "kube.traefik")]
    pub kube_traefik: bool,
    /// Enables the Ingress-Kubernetes-Configurator
    #[structopt(long = "kube.ingress")]
    pub kube_ingress: bool,
    /// Enables the File-Configurator and reads the config from
    /// the provided File/Directory
    #[structopt(long = "file-conf")]
    pub file: Option<String>,
    /// Enables the Webserver-Entrypoint
    #[structopt(long = "webserver")]
    pub webserver: Option<u32>,
    /// Enables the Tunneler-Entrypoint
    #[structopt(long = "tunneler")]
    pub tunneler: bool,
}

impl Options {
    /// If at least one of the Kubernetes Options has been enabled
    pub fn is_kubernetes_enabled(&self) -> bool {
        self.kube_ingress || self.kube_traefik
    }
}
