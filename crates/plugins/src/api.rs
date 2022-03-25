use wasmer::{imports, Function, ImportObject, Store};

mod env;
pub use env::{PluginContext, PluginEnv};

mod acceptor;
mod body;
mod header;
mod path;

const REQUEST_RESSOURCE_ID: i32 = 0;
const RESPONSE_RESSOURCE_ID: i32 = 1;

pub fn get_imports(store: &Store, exec_env: &PluginEnv) -> ImportObject {
    imports! {
        "env" => {
            "log_error" => Function::new_native_with_env(store, exec_env.clone(), log_error),
            "get_config" => Function::new_native_with_env(store, exec_env.clone(), get_config),
            "get_config_str" => Function::new_native_with_env(store, exec_env.clone(), get_config_str),
            "action_get_method" => Function::new_native_with_env(store, exec_env.clone(), get_method),
            "action_get_status_code" => Function::new_native_with_env(store, exec_env.clone(), get_status_code),
            "action_get_path"  => Function::new_native_with_env(store, exec_env.clone(), path::get_path),
            "action_set_path" => Function::new_native_with_env(store, exec_env.clone(), path::set_path),
            "action_has_header" => Function::new_native_with_env(store, exec_env.clone(), header::has_header),
            "action_get_header" => Function::new_native_with_env(store, exec_env.clone(), header::get_header),
            "action_set_header" => Function::new_native_with_env(store, exec_env.clone(), header::set_header_text),
            "action_get_body" => Function::new_native_with_env(store, exec_env.clone(), body::get_body),
            "action_set_body" => Function::new_native_with_env(store, exec_env.clone(), body::set_body),
            "acceptor_new_con" => Function::new_native_with_env(store, exec_env.clone(), acceptor::new_con),
            "acceptor_has_send" => Function::new_native_with_env(store, exec_env.clone(), acceptor::has_send),
            "acceptor_send" => Function::new_native_with_env(store, exec_env.clone(), acceptor::send),
            "acceptor_recv" => Function::new_native_with_env(store, exec_env.clone(), acceptor::recv),
        }
    }
}

pub fn log_error(env: &PluginEnv, buffer_ptr: i32, buffer_length: i32) {
    let slice = env.get_memory_slice(buffer_ptr as usize, buffer_length as usize);
    let content = std::str::from_utf8(slice.as_slice())
        .expect("The Logged content should always be a valid String");

    tracing::error!("{}", content);
}

pub fn get_config(env: &PluginEnv, target_addr: i32) {
    let config = env.config.as_ref();
    if config.is_empty() {
        return;
    }

    let config_size = config.len();

    let mem = env.get_memory_slice(target_addr as usize, config_size);
    mem.as_mut_slice().copy_from_slice(config);
}

pub fn get_config_str(env: &PluginEnv, target_addr: i32) {
    let config_str = match &env.context {
        PluginContext::Config { config_str } => config_str,
        _ => return,
    };

    env.set_string(target_addr, config_str);
}

pub fn get_method(env: &PluginEnv) -> i32 {
    match &env.context {
        PluginContext::ActionApplyReq { request, .. } => request.method().wasm_serialize(),
        _ => -1,
    }
}

pub fn get_status_code(env: &PluginEnv) -> i32 {
    match &env.context {
        PluginContext::ActionApplyResp { response, .. } => response.status_code().wasm_serialize(),
        _ => -1,
    }
}
