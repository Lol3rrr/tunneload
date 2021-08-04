use argser::argser;

use super::{AutoTLSOpts, KubernetesOpts, TunnelerOpts, WebserverOpts};

/// The Command-Line options provided by the Load-Balancer
#[argser]
#[derive(Debug)]
pub struct Options {
    /// The Kubernetes related options
    #[argser(subcategory)]
    pub kubernetes: KubernetesOpts,

    /// Enables the File-Configurator and reads the config from
    /// the provided File/Directory
    #[argser(rename("file-conf"), default)]
    pub file: Option<String>,

    /// The Webserver related options
    #[argser(subcategory)]
    pub webserver: WebserverOpts,
    /// The Tunneler related options
    #[argser(subcategory)]
    pub tunneler: TunnelerOpts,

    /// Enables the Metrics endpoint
    #[argser(rename("metrics"), default)]
    pub metrics: Option<u32>,

    /// Enables the internal Dashboard
    #[argser(default)]
    pub dashboard: bool,

    /// Enables the Plugins and loads them in from the
    /// given File/Directory
    #[argser(rename("plugins"), default)]
    pub plugin_file: Option<String>,

    /// The Auto-TLS related options
    #[argser(subcategory)]
    pub auto_tls: AutoTLSOpts,
}
