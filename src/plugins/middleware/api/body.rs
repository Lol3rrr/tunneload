use crate::plugins::middleware::{ExecutionEnv, MiddlewareOp};

pub fn get_body(env: &ExecutionEnv, target_address: i32) {
    let mem = unsafe { env.memory.load().unwrap().data_unchecked_mut() };

    let body = match env.body() {
        Some(b) => b,
        None => panic!("Could not load a body"),
    };

    let start = target_address as usize;
    let end = start + body.len();

    mem[start..end].copy_from_slice(body);
}

pub fn set_body(env: &ExecutionEnv, address: i32, length: i32) {
    let mem = unsafe { env.memory.load().unwrap().data_unchecked() };

    let mut buffer = Vec::with_capacity(length as usize);

    let start = address as usize;
    let end = start + length as usize;

    buffer.extend_from_slice(&mem[start..end]);

    env.add_op(MiddlewareOp::SetBody(buffer));
}
