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
}
