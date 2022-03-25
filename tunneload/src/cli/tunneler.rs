use argser::argser;

/// All the Tunneler specific Options
#[argser]
#[derive(Debug)]
pub struct TunnelerOpts {
    /// Sets the Address of the Tunneler-Server to conect to
    #[argser(default_func(default_addr))]
    pub addr: String,
    /// Sets the external Port the Tunneler-Entrypoint should
    /// connect to
    #[argser(default_func(default_port))]
    pub port: u32,
    /// Sets the public Port on the Tunneler-Server
    #[argser(default_func(default_public_port))]
    pub public_port: u16,
    /// Sets the key-file Path to use
    #[argser(default_func(default_key))]
    pub key: String,
    /// Enables TLS for the Option
    #[argser(default)]
    pub tls: bool,
}

fn default_addr() -> String {
    "localhost".to_string()
}

fn default_port() -> u32 {
    8081
}

fn default_public_port() -> u16 {
    80
}

fn default_key() -> String {
    let mut key_path =
        dirs::home_dir().expect("We should always be able to get the Home Directory");
    key_path.push(".tunneler");
    key_path.push("key");
    key_path
        .as_path()
        .to_str()
        .expect("The Path should also be a valid Path")
        .to_string()
}
