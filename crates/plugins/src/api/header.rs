use crate::action::MiddlewareOp;

use super::{
    env::{PluginContext, PluginEnv},
    REQUEST_RESSOURCE_ID, RESPONSE_RESSOURCE_ID,
};

pub fn set_header_text(
    env: &PluginEnv,
    ressource: i32,
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
                .expect("Obtaining the Lock should never fail")
                .push(MiddlewareOp::SetHeader(ressource, key, value));
        }
        _ => {}
    };
}

pub fn has_header(env: &PluginEnv, ressource: i32, key_addr: i32, key_length: i32) -> i32 {
    let key = env.load_string(key_addr, key_length);

    let headers = match ressource {
        REQUEST_RESSOURCE_ID => match env.get_request() {
            Some(resp) => resp.headers(),
            None => panic!("Attempting to load Headers from Request, but Request is not set"),
        },
        RESPONSE_RESSOURCE_ID => match env.get_response() {
            Some(resp) => resp.headers(),
            None => panic!("Attempting to load Headers from Response, but Response is not set"),
        },
        _ => return 0,
    };

    match headers.get(key) {
        Some(_) => 1,
        None => 0,
    }
}

pub fn get_header(
    env: &PluginEnv,
    ressource: i32,
    target_addr: i32,
    key_addr: i32,
    key_length: i32,
) -> i32 {
    let key = env.load_string(key_addr, key_length);

    let headers = match ressource {
        REQUEST_RESSOURCE_ID => match env.get_request() {
            Some(req) => req.headers(),
            None => panic!("Attempting to load Headers from Request, but Request is not set"),
        },
        RESPONSE_RESSOURCE_ID => match env.get_response() {
            Some(resp) => resp.headers(),
            None => panic!("Attempting to load Headers from Response, but Response is not set"),
        },
        _ => return 0,
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
