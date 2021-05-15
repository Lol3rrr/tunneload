use crate::cli::KubernetesOpts;
use crate::cli::TunnelerOpts;
use crate::cli::WebserverOpts;

use structopt::StructOpt;

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
}
