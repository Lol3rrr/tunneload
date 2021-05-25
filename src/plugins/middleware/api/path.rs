use crate::plugins::middleware::{ExecutionEnv, MiddlewareOp};

pub fn get_path(env: &ExecutionEnv, target_addr: i32) {
    let req = match env.get_request() {
        Some(r) => r,
        None => panic!("Attempted to load Path, when Request was not set"),
    };

    let path = req.path();
    let data = path.as_bytes();

    let data_slice = unsafe {
        env.memory
            .load()
            .expect("Expected memory to be loaded")
            .data_unchecked_mut()
    };

    let start = target_addr as usize;
    let end = (target_addr as usize) + data.len();
    data_slice[start..end].copy_from_slice(&data);
}

pub fn set_path(env: &ExecutionEnv, path_addr: i32, path_length: i32) {
    let n_path = env.load_string(path_addr, path_length).unwrap();

    env.add_op(MiddlewareOp::SetPath(n_path));
}
