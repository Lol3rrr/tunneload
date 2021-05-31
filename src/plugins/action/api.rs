use wasmer::{imports, Function, ImportObject, Store};

use super::ExecutionEnv;

mod body;
mod header;
mod path;

pub fn get_imports(store: &Store, exec_env: &ExecutionEnv) -> ImportObject {
    imports! {
        "env" => {
            "get_config" => Function::new_native_with_env(store, exec_env.clone(), get_config),
            "get_config_str" => Function::new_native_with_env(store, exec_env.clone(), get_config_str),
            "get_method" => Function::new_native_with_env(store, exec_env.clone(), get_method),
            "get_status_code" => Function::new_native_with_env(store, exec_env.clone(), get_status_code),
            "get_path"  => Function::new_native_with_env(store, exec_env.clone(), path::get_path),
            "set_path" => Function::new_native_with_env(store, exec_env.clone(), path::set_path),
            "set_header_text" => Function::new_native_with_env(store, exec_env.clone(), header::set_header_text),
            "has_header" => Function::new_native_with_env(store, exec_env.clone(), header::has_header),
            "get_header" => Function::new_native_with_env(store, exec_env.clone(), header::get_header),
            "get_body" => Function::new_native_with_env(store, exec_env.clone(), body::get_body),
            "set_body" => Function::new_native_with_env(store, exec_env.clone(), body::set_body)
        }
    }
}

pub fn get_config(env: &ExecutionEnv, target_addr: i32) {
    let config = env.config.as_ref();
    if config.is_empty() {
        return;
    }

    let config_size = config.len();

    let mem = env.get_mut_memory_slice(target_addr as usize, config_size);
    mem.copy_from_slice(&config);
}

pub fn get_config_str(env: &ExecutionEnv, target_addr: i32) {
    let config_str = match &env.get_config_str() {
        Some(s) => s,
        None => return,
    };

    env.set_string(target_addr, &config_str);
}

pub fn get_method(env: &ExecutionEnv) -> i32 {
    match env.get_request() {
        Some(req) => req.method().wasm_serialize(),
        None => -1,
    }
}

pub fn get_status_code(env: &ExecutionEnv) -> i32 {
    match env.get_response() {
        Some(resp) => resp.status_code().wasm_serialize(),
        None => -1,
    }
}
