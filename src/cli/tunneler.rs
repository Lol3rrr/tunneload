use argser::argser;

/// All the Tunneler specific Options
#[argser]
#[derive(Debug)]
pub struct TunnelerOpts {
    /// Enables the Tunneler-Entrypoint
    #[argser(rename("enabled"), default)]
    pub enabled: bool,
    /// Sets the Address of the Tunneler-Server to conect to
    #[argser(rename("addr"), default)]
    pub tunneler_addr: Option<String>,
    /// Sets the external Port the Tunneler-Entrypoint should
    /// connect to
    #[argser(rename("port"), default)]
    pub tunneler_port: Option<u32>,
    /// Sets the key-file Path to use
    #[argser(rename("key"), default)]
    pub tunneler_key: Option<String>,

    /// Enables the Tunneler-Entrypoint with TLS
    #[argser(rename("tls"), default)]
    pub tls_enabled: bool,
    /// Sets the Address of the Tunneler-Server to conect to
    #[argser(rename("addr.tls"), default)]
    pub tls_tunneler_addr: Option<String>,
    /// Sets the external Port the Tunneler-Entrypoint should
    /// connect to
    #[argser(rename("port.tls"), default)]
    pub tls_tunneler_port: Option<u32>,
    /// Sets the key-file Path to use
    #[argser(rename("key.tls"), default)]
    pub tls_tunneler_key: Option<String>,
}

impl TunnelerOpts {
    /// Returns the configuration for the Non-TLS Tunneler
    /// with default values if there are parts not set
    ///
    /// Format:
    /// Key-File, Address, Port
    pub fn get_normal_with_defaults(&self) -> (String, String, u32) {
        let key_file = match self.tunneler_key.as_ref() {
            Some(val) => val.to_owned(),
            None => {
                let mut key_path = dirs::home_dir().unwrap();
                key_path.push(".tunneler");
                key_path.push("key");
                key_path.as_path().to_str().unwrap().to_string()
            }
        };

        let addr = match self.tunneler_addr.as_ref() {
            Some(val) => val.to_owned(),
            None => "localhost".to_owned(),
        };
        let port = self.tunneler_port.unwrap_or(8081);

        (key_file, addr, port)
    }

    /// Checks if the Normal (non-TLS) version of the Tunneler
    /// is enabled
    pub fn is_normal_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the configuration for the TLS Tunneler
    /// with default values if there are parts not set
    ///
    /// Format:
    /// Key-File, Address, Port
    pub fn get_tls_with_defaults(&self) -> (String, String, u32) {
        let key_file = match self.tls_tunneler_key.as_ref() {
            Some(val) => val.to_owned(),
            None => {
                let mut key_path = dirs::home_dir().unwrap();
                key_path.push(".tunneler");
                key_path.push("key");
                key_path.as_path().to_str().unwrap().to_string()
            }
        };

        let addr = match self.tls_tunneler_addr.as_ref() {
            Some(val) => val.to_owned(),
            None => "localhost".to_owned(),
        };
        let port = self.tls_tunneler_port.unwrap_or(8081);

        (key_file, addr, port)
    }

    /// Checks if the TLS version is enabled
    pub fn is_tls_enabled(&self) -> bool {
        self.tls_enabled
    }
}
