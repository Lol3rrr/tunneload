use structopt::StructOpt;

use super::{AutoTLSOpts, KubernetesOpts, TunnelerOpts, WebserverOpts};

/// The Command-Line options provided by the Load-Balancer
#[derive(Debug, StructOpt)]
pub struct Options {
    /// The Kubernetes related options
    #[structopt(flatten)]
    pub kubernetes: KubernetesOpts,

    /// Enables the File-Configurator and reads the config from
    /// the provided File/Directory
    #[structopt(long = "file-conf")]
    pub file: Option<String>,

    /// The Webserver related options
    #[structopt(flatten)]
    pub webserver: WebserverOpts,
    /// The Tunneler related options
    #[structopt(flatten)]
    pub tunneler: TunnelerOpts,

    /// Enables the Metrics endpoint
    #[structopt(long = "metrics")]
    pub metrics: Option<u32>,

    /// Enables the internal Dashboard
    #[structopt(long = "dashboard")]
    pub dashboard: bool,

    /// Enables the Plugins and loads them in from the
    /// given File/Directory
    #[structopt(long = "plugins")]
    pub plugin_file: Option<String>,

    /// The Auto-TLS related options
    #[structopt(flatten)]
    pub auto_tls: AutoTLSOpts,
}
