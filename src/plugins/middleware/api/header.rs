use crate::plugins::middleware::{ExecutionEnv, MiddlewareOp};

pub fn set_header_text(
    env: &ExecutionEnv,
    key_addr: i32,
    key_length: i32,
    value_addr: i32,
    value_length: i32,
) {
    let key = env.load_string(key_addr, key_length).unwrap();
    let value = env.load_string(value_addr, value_length).unwrap();

    env.add_op(MiddlewareOp::SetHeader(key, value));
}

pub fn has_header(env: &ExecutionEnv, key_addr: i32, key_length: i32) -> i32 {
    let key = env.load_string(key_addr, key_length).unwrap();

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

pub fn get_header(env: &ExecutionEnv, target_addr: i32, key_addr: i32, key_length: i32) -> i32 {
    let key = env.load_string(key_addr, key_length).unwrap();

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
