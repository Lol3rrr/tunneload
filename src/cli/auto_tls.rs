use argser::argser;

#[argser]
#[derive(Debug)]
pub struct FileOptions {
    /// The File from which the Cluster-Configuration should be loaded
    #[argser(rename("path"), default)]
    pub path: Option<String>,

    /// The Directory where the generated Certificates will be stored
    #[argser(rename("dir"), default_func(default_file_dir))]
    pub directory: String,
}

#[argser]
#[derive(Debug)]
pub struct ClusterOptions {
    /// The Port used by the Tunneload instances to communicate with each other
    #[argser(rename("port"), default_func(default_cluster_port))]
    pub port: u16,
}

/// The Auto-TLS specific Options
#[argser]
#[derive(Debug)]
pub struct AutoTLSOpts {
    /// Enable Auto-TLS
    #[argser(rename("enable"), default)]
    pub auto_tls_enabled: bool,

    /// Whether or not to use the Production Endpoint for Let's-Encrypt
    #[argser(rename("production"), default)]
    pub auto_tls_production: bool,

    /// The Kubernetes service used to discover all the other Tunneload
    /// instances
    #[argser(rename("service"), default)]
    pub kubernetes_service: Option<String>,
    /// The Kubernetes-Namespace to use
    #[argser(rename("namespace"), default_func(default_namespace))]
    pub kubernetes_namespace: String,

    /// The File Options
    #[argser(subcategory)]
    pub file: FileOptions,

    /// The Port used by the Tunneload instances to communicate with each other
    #[argser(subcategory)]
    pub cluster: ClusterOptions,
}

fn default_namespace() -> String {
    "default".to_string()
}
fn default_file_dir() -> String {
    "certs/".to_string()
}
fn default_cluster_port() -> u16 {
    8375
}
