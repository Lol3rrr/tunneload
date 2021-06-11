use crate::plugins::action::MiddlewareOp;

use super::env::{PluginContext, PluginEnv};

pub fn get_path(env: &PluginEnv, target_addr: i32) {
    let req = match env.get_request() {
        Some(r) => r,
        None => panic!("Attempted to load Path, when Request was not set"),
    };

    let path = req.path();

    env.set_string(target_addr, path);
}

pub fn set_path(env: &PluginEnv, path_addr: i32, path_length: i32) {
    let n_path = env.load_string(path_addr, path_length);

    match &env.context {
        PluginContext::ActionApplyReq { ops, .. } => {
            ops.lock().unwrap().push(MiddlewareOp::SetPath(n_path));
        }
        _ => {}
    };
}
