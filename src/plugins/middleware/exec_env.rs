use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use stream_httparse::{Request, Response};
use wasmer::{Memory, WasmerEnv};

#[derive(Debug)]
pub enum MiddlewareOp {
    SetPath(String),
    SetHeader(String, String),
    SetBody(Vec<u8>),
}

#[derive(Debug, Clone, WasmerEnv)]
pub struct ExecutionEnv {
    pub config: Arc<Vec<u8>>,
    config_str: Arc<Option<String>>,
    request: Arc<Option<&'static Request<'static>>>,
    response: Arc<Option<&'static Response<'static>>>,
    pub memory: Arc<ArcSwap<Option<&'static Memory>>>,
    pub ops: Arc<Mutex<Vec<MiddlewareOp>>>,
}

impl ExecutionEnv {
    pub fn new(
        request: Option<&Request<'_>>,
        response: Option<&Response<'_>>,
        config: Arc<Vec<u8>>,
    ) -> Self {
        let request = Arc::new(unsafe {
            std::mem::transmute::<Option<&Request<'_>>, Option<&'static Request<'_>>>(request)
        });
        let response = Arc::new(unsafe {
            std::mem::transmute::<Option<&Response<'_>>, Option<&'static Response<'_>>>(response)
        });

        Self {
            config,
            config_str: Arc::new(None),
            request,
            response,
            memory: Arc::new(ArcSwap::new(Arc::new(None))),
            ops: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_config_str(&mut self, config: String) {
        self.config_str = Arc::new(Some(config));
    }
    pub fn get_config_str(&self) -> &Option<String> {
        self.config_str.as_ref()
    }

    pub fn get_request(&self) -> &Option<&Request<'_>> {
        self.request.as_ref()
    }
    pub fn get_response(&self) -> &Option<&Response<'_>> {
        self.response.as_ref()
    }

    pub unsafe fn store_mem_ref<'a>(&self, mem: &'a Memory) {
        let tmp = std::mem::transmute::<&'a Memory, &'static Memory>(mem);

        self.memory.store(Arc::new(Some(tmp)));
    }

    pub fn add_op(&self, op: MiddlewareOp) {
        self.ops.lock().unwrap().push(op);
    }

    pub fn load_string(&self, addr: i32, length: i32) -> Option<String> {
        let mem = unsafe { self.memory.load().unwrap().data_unchecked() };

        let start = addr as usize;
        let end = (addr + length) as usize;

        let mut bytes: Vec<u8> = Vec::with_capacity(length as usize);
        bytes.extend_from_slice(&mem[start..end]);

        let result = String::from_utf8(bytes).unwrap();

        Some(result)
    }

    pub fn set_string(&self, target: i32, value: &str) {
        let mem = unsafe { self.memory.load().unwrap().data_unchecked_mut() };

        let raw_data = value.as_bytes();

        let start = target as usize;
        let end = start + raw_data.len();

        &mem[start..end].copy_from_slice(&raw_data);
    }

    pub fn body(&self) -> Option<&[u8]> {
        if let Some(req) = self.request.as_ref() {
            return Some(req.body());
        }
        if let Some(resp) = self.response.as_ref() {
            return Some(resp.body());
        }

        None
    }

    pub fn get_memory_slice(&self, start: usize, size: usize) -> &[u8] {
        let mem = unsafe { self.memory.load().unwrap().data_unchecked() };
        &mem[start..start + size]
    }
    pub fn get_mut_memory_slice(&self, start: usize, size: usize) -> &mut [u8] {
        let mem = unsafe { self.memory.load().unwrap().data_unchecked_mut() };
        &mut mem[start..start + size]
    }
}
