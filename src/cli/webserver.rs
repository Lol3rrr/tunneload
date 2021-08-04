use argser::argser;

/// All the Webserver specific options
#[argser]
#[derive(Debug)]
pub struct WebserverOpts {
    /// Enables the Non-TLS webserver on the given
    /// port
    #[argser(rename("port"), default)]
    pub port: Option<u32>,
    /// Enables the TLS webserver on the given port
    #[argser(rename("tls"), default)]
    pub tls_port: Option<u32>,
}
