use crate::cli::KubernetesOpts;
use crate::cli::TunnelerOpts;
use crate::cli::WebserverOpts;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(flatten)]
    pub kubernetes: KubernetesOpts,

    /// Enables the File-Configurator and reads the config from
    /// the provided File/Directory
    #[structopt(long = "file-conf")]
    pub file: Option<String>,

    // All the Acceptors
    #[structopt(flatten)]
    pub webserver: WebserverOpts,
    #[structopt(flatten)]
    pub tunneler: TunnelerOpts,

    /// Enables the Metrics endpoint
    #[structopt(long = "metrics")]
    pub metrics: Option<u32>,
}
