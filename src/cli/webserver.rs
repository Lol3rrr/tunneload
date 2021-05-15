use structopt::StructOpt;

/// All the Webserver specific options
#[derive(Debug, StructOpt)]
pub struct WebserverOpts {
    /// Enables the Non-TLS webserver on the given
    /// port
    #[structopt(long = "webserver")]
    pub port: Option<u32>,
    /// Enables the TLS webserver on the given port
    #[structopt(long = "webserver.tls")]
    pub tls_port: Option<u32>,
}
