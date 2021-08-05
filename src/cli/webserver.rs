use argser::argser;

/// All the Webserver specific options
#[argser]
#[derive(Debug)]
pub struct WebserverOpts {
    /// Enables the Non-TLS webserver on the given
    /// port
    #[argser(default)]
    pub port: u32,
    /// Enables the TLS webserver on the given port
    #[argser(default)]
    pub tls: bool,
}
