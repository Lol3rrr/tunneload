use crate::plugins::action::MiddlewareOp;

use super::env::{PluginContext, PluginEnv};

pub fn set_header_text(
    env: &PluginEnv,
    key_addr: i32,
    key_length: i32,
    value_addr: i32,
    value_length: i32,
) {
    let key = env.load_string(key_addr, key_length);
    let value = env.load_string(value_addr, value_length);

    match &env.context {
        PluginContext::ActionApplyReq { ops, .. } | PluginContext::ActionApplyResp { ops, .. } => {
            ops.lock()
                .unwrap()
                .push(MiddlewareOp::SetHeader(key, value));
        }
        _ => {}
    };
}

pub fn has_header(env: &PluginEnv, key_addr: i32, key_length: i32) -> i32 {
    let key = env.load_string(key_addr, key_length);

    let headers = match env.get_request() {
        Some(req) => req.headers(),
        None => match env.get_response() {
            Some(resp) => resp.headers(),
            None => panic!("Attempting to load Headers, when no Request or Response was specified"),
        },
    };

    match headers.get(key) {
        Some(_) => 1,
        None => 0,
    }
}

pub fn get_header(env: &PluginEnv, target_addr: i32, key_addr: i32, key_length: i32) -> i32 {
    let key = env.load_string(key_addr, key_length);

    let headers = match env.get_request() {
        Some(req) => req.headers(),
        None => match env.get_response() {
            Some(resp) => resp.headers(),
            None => panic!("Attempting to load Headers, when no Request or Response was specified"),
        },
    };

    match headers.get(key) {
        Some(value) => {
            let value_str = value.to_string();
            env.set_string(target_addr, &value_str);

            value_str.len() as i32
        }
        None => 0,
    }
}
