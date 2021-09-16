#![warn(missing_docs)]
//! This contains all the needed Parts for using WASM-Modules as plugins
//! for different parts of the Load-Balancer to make it more modular
//! and add features without having to work on the general source code

mod acceptor;
pub use acceptor::{AcceptorPluginInstance, PluginAcceptor};
mod action;
pub use action::ActionPluginInstance;

use std::{convert::TryInto, sync::Arc};

mod loader;
pub use loader::Loader;

mod api;

use serde::{ser::SerializeMap, Serialize, Serializer};
use wasmer::{Instance, Module, Store};

use general_traits::ConfigItem;

use self::api::PluginEnv;

/// This defines a simple interface to create actual Plugin-Instances, with config,
/// from a basic loaded WASM-Plugin
pub trait InstantiatePlugin {
    /// Actually creates the new Instance of the Plugin with the given Configuration/data
    fn instantiate(name: String, store: Store, module: Module, config: Option<Vec<u8>>) -> Self;
}

#[tracing::instrument(skip(store, exec_env, module))]
fn start_instance(store: &Store, exec_env: &PluginEnv, module: &Module) -> Option<Instance> {
    let import_objects = api::get_imports(store, exec_env);
    let instance = match Instance::new(module, &import_objects) {
        Ok(i) => i,
        Err(e) => {
            tracing::error!("Creating WASM-Instance: {:?}", e);
            return None;
        }
    };

    Some(instance)
}

/// This represents a single Plugin that is loaded from an external
/// WASM based plugin/module
#[derive(Debug, Clone)]
pub struct Plugin {
    name: String,
    store: Store,
    module: wasmer::Module,
}

impl Plugin {
    fn parse_initial_config(store: &Store, module: &Module, config_str: String) -> Option<Vec<u8>> {
        let exec_env = PluginEnv::new(
            Arc::new(Vec::new()),
            api::PluginContext::Config {
                config_str: config_str.clone(),
            },
        );

        let instance = start_instance(store, &exec_env, module)?;

        let instance_parse_config = match instance
            .exports
            .get_native_function::<i32, i32>("parse_config")
        {
            Ok(f) => f,
            Err(_) => return None,
        };

        let raw_config_ptr = match instance_parse_config.call(config_str.len() as i32) {
            Ok(p) => p,
            Err(_) => return None,
        };

        let memory = instance.exports.get_memory("memory").unwrap();

        let raw_config_ptr = raw_config_ptr as usize;
        let raw_config_ptr_data =
            unsafe { &memory.data_unchecked()[raw_config_ptr..raw_config_ptr + 8] };

        let config_ptr = i32::from_be_bytes(raw_config_ptr_data[0..4].try_into().unwrap());
        let config_size = i32::from_be_bytes(raw_config_ptr_data[4..8].try_into().unwrap());

        let config_ptr = config_ptr as usize;
        let config_size = config_size as usize;

        let data =
            unsafe { &memory.data_unchecked()[config_ptr..config_ptr + config_size] }.to_vec();

        Some(data)
    }

    /// Attempts to create a new WASM module Plugin using the given
    /// Data as the actual wasm
    pub fn new(name: String, wasm_data: &[u8]) -> Option<Self> {
        let store = Store::default();
        let module = Module::from_binary(&store, wasm_data).unwrap();

        Some(Self {
            name,
            store,
            module,
        })
    }

    /// Creates an actual Instance of the Plugin with the given Config
    pub fn create_instance<I>(&self, config_str: String) -> Option<I>
    where
        I: InstantiatePlugin,
    {
        let store = self.store.clone();
        let module = self.module.clone();

        let config = Self::parse_initial_config(&store, &module, config_str);

        Some(I::instantiate(self.name.clone(), store, module, config))
    }

    fn check_for_acceptor(store: &Store, module: &Module) -> bool {
        let exec_env = PluginEnv::new(Arc::new(Vec::new()), api::PluginContext::TypeCheck);

        let instance = start_instance(store, &exec_env, module).unwrap();

        instance.exports.get_function("accept").is_ok()
    }

    /// Checks if the given Plugin is an Acceptor
    pub fn is_acceptor(&self) -> bool {
        Self::check_for_acceptor(&self.store, &self.module)
    }
}

impl ConfigItem for Plugin {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Serialize for Plugin {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;

        map.serialize_entry("name", &self.name)?;

        map.end()
    }
}
