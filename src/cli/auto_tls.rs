use structopt::StructOpt;

/// The Auto-TLS specific Options
#[derive(Debug, StructOpt)]
pub struct AutoTLSOpts {
    /// Enable Auto-TLS
    #[structopt(long = "auto-tls.enable")]
    pub auto_tls_enabled: bool,

    /// Whether or not to use the Production Endpoint for Let's-Encrypt
    #[structopt(long = "auto-tls.production")]
    pub auto_tls_production: bool,

    /// The Kubernetes service used to discover all the other Tunneload
    /// instances
    #[structopt(long = "auto-tls.service")]
    pub kubernetes_service: Option<String>,

    /// The File from which the Cluster-Configuration should be loaded
    #[structopt(long = "auto-tls.file.path")]
    pub file_path: Option<String>,

    /// The Port used by the Tunneload instances to communicate with each other
    #[structopt(long = "auto-tls.cluster.port", default_value = "8375")]
    pub cluster_port: u16,
}
