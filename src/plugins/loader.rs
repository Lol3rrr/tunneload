use std::path::Path;

use super::{acceptor::PluginAcceptor, AcceptorPluginInstance, Plugin};

/// A Configurator, which is responsible for loading all the
/// Plugins into Tunneload
pub struct Loader {
    path: String,
}

impl Loader {
    /// Creates a new Plugin-Loader with the given Path-String
    /// as the source of Plugins to load
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Attempts to load all the Plugins from the configured Path
    pub fn load_plugins(&self) -> Vec<Plugin> {
        load_plugins(&self.path)
    }

    /// Attempts to load all the Acceptor-Plugins from the configured
    /// Path
    pub fn load_acceptors(&self) -> Vec<PluginAcceptor> {
        let plugins = self.load_plugins();

        let mut result = Vec::new();
        for tmp in plugins {
            if !tmp.is_acceptor() {
                continue;
            }

            let acceptor_instance: AcceptorPluginInstance =
                tmp.create_instance("".to_owned()).unwrap();
            let acceptor = PluginAcceptor::new(acceptor_instance);

            result.push(acceptor);
        }

        result
    }
}

pub fn load_plugins(path: &str) -> Vec<Plugin> {
    let path = Path::new(path);
    let metadata = std::fs::metadata(path).unwrap();
    if metadata.is_file() {
        let raw_file_name = path.file_name().unwrap().to_str().unwrap();
        let (file_name, file_ending) = match raw_file_name.split_once('.') {
            Some(s) => s,
            None => {
                return Vec::new();
            }
        };

        if file_ending != "wasm" {
            return Vec::new();
        }

        let content = match std::fs::read(path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Reading File: {:?}", e);
                return Vec::new();
            }
        };

        let plugin = match Plugin::new(file_name.to_owned(), &content) {
            Some(p) => p,
            None => {
                tracing::error!("Loading WASM-Plugin");
                return Vec::new();
            }
        };

        vec![plugin]
    } else {
        let mut result = Vec::new();

        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let raw_path = entry.path();
            let path_str = raw_path.to_str().unwrap();
            result.extend(load_plugins(path_str));
        }
        result
    }
}
