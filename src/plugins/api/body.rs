use crate::plugins::action::MiddlewareOp;

use super::env::{PluginContext, PluginEnv};

pub fn get_body(env: &PluginEnv, ressource: i32, target_address: i32) {
    let body = match env.context.body(ressource) {
        Some(b) => b,
        None => panic!("Could not load a body"),
    };

    let start = target_address as usize;

    let mem = env.get_memory_slice(start, body.len());

    mem.as_mut_slice().copy_from_slice(body);
}

pub fn set_body(env: &PluginEnv, ressource: i32, address: i32, length: i32) {
    let length = length as usize;
    let mut buffer = Vec::with_capacity(length);

    let start = address as usize;

    let mem = env.get_memory_slice(start, length);

    buffer.extend_from_slice(mem.as_slice());

    match &env.context {
        PluginContext::ActionApplyReq { ops, .. } | PluginContext::ActionApplyResp { ops, .. } => {
            ops.lock()
                .unwrap()
                .push(MiddlewareOp::SetBody(ressource, buffer));
        }
        _ => {}
    };
}
