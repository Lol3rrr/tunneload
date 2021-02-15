use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    /// Enable/Disable the Kubernetes-Configurator
    #[structopt(long = "kube-conf")]
    pub kubernetes: bool,
    /// Enables the File-Configurator and reads the config from
    /// the provided File/Directory
    #[structopt(long = "file-conf")]
    pub file: Option<String>,
}
