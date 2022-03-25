use std::path::Path;

use general::{Group, Name};

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

            let acceptor_instance: AcceptorPluginInstance = match tmp.create_instance("".to_owned())
            {
                Some(i) => i,
                None => continue,
            };
            let acceptor = PluginAcceptor::new(acceptor_instance);

            result.push(acceptor);
        }

        result
    }
}

pub fn load_plugins(path: &str) -> Vec<Plugin> {
    let path = Path::new(path);
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };
    if metadata.is_file() {
        let raw_file_name = path
            .file_name()
            .expect("The File should have a Name")
            .to_str()
            .expect("The FileName shuold be a valid String");
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

        let raw_name = file_name;
        let name = Name::new(raw_name, Group::File {});

        let plugin = match Plugin::new(name, &content) {
            Some(p) => p,
            None => {
                tracing::error!("Loading WASM-Plugin");
                return Vec::new();
            }
        };

        vec![plugin]
    } else {
        let mut result = Vec::new();

        let entry_list = match std::fs::read_dir(path) {
            Ok(el) => el,
            Err(_) => return Vec::new(),
        };
        for entry in entry_list.into_iter().filter_map(|re| re.ok()) {
            let raw_path = entry.path();
            let path_str = raw_path
                .to_str()
                .expect("The FilePath should be a valid String");
            result.extend(load_plugins(path_str));
        }
        result
    }
}
