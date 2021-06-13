use std::sync::{Arc, Mutex};

use stream_httparse::{Request, Response};
use wasmer::{HostEnvInitError, Instance, Memory, WasmerEnv};

use crate::plugins::action::MiddlewareOp;

#[derive(Debug, Clone)]
pub struct PluginEnv {
    memory: Arc<once_cell::sync::OnceCell<Memory>>,
    pub config: Arc<Vec<u8>>,
    pub context: PluginContext,
}

#[derive(Debug, Clone)]
pub enum PluginContext {
    Config {
        config_str: String,
    },
    ActionApplyReq {
        request: Arc<&'static Request<'static>>,
        ops: Arc<Mutex<Vec<MiddlewareOp>>>,
    },
    ActionApplyResp {
        response: Arc<&'static Response<'static>>,
        ops: Arc<Mutex<Vec<MiddlewareOp>>>,
    },
}

impl WasmerEnv for PluginEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        let memory = instance.exports.get_memory("memory").unwrap();
        if self.memory.set(memory.clone()).is_err() {
            log::error!("Memory was already set for the Plugin-Environment");
        }
        Ok(())
    }
}

impl PluginEnv {
    pub fn new(config: Arc<Vec<u8>>, context: PluginContext) -> Self {
        Self {
            memory: Arc::new(once_cell::sync::OnceCell::new()),
            config,
            context,
        }
    }

    pub fn get_memory_slice(&self, start: usize, size: usize) -> &[u8] {
        let mem = unsafe { self.memory.get().unwrap().data_unchecked() };
        &mem[start..start + size]
    }
    #[allow(clippy::mut_from_ref)]
    pub fn get_mut_memory_slice(&self, start: usize, size: usize) -> &mut [u8] {
        let mem = unsafe { self.memory.get().unwrap().data_unchecked_mut() };
        &mut mem[start..start + size]
    }

    pub fn load_string(&self, target: i32, target_length: i32) -> String {
        let length = target_length as usize;
        let start = target as usize;

        let mem = self.get_memory_slice(start, length);

        let mut bytes: Vec<u8> = Vec::with_capacity(length);
        bytes.extend_from_slice(&mem);

        String::from_utf8(bytes).unwrap()
    }

    pub fn set_string(&self, target: i32, data: &str) {
        let raw_data = data.as_bytes();

        let start = target as usize;

        let mem = self.get_mut_memory_slice(start, raw_data.len());
        mem.copy_from_slice(&raw_data);
    }

    pub fn get_request(&self) -> Option<&Request<'static>> {
        match &self.context {
            PluginContext::ActionApplyReq { request, .. } => Some(request.as_ref()),
            _ => None,
        }
    }
    pub fn get_response(&self) -> Option<&Response<'static>> {
        match &self.context {
            PluginContext::ActionApplyResp { response, .. } => Some(response.as_ref()),
            _ => None,
        }
    }
}

impl PluginContext {
    // TODO
    // This function should be fixed/split out to allow the
    // consumer to chose whether to load the Request-Body
    // or the Response-Body
    pub fn body(&self) -> Option<&[u8]> {
        match self {
            Self::ActionApplyReq { request, .. } => Some(request.body()),
            Self::ActionApplyResp { response, .. } => Some(response.body()),
            _ => None,
        }
    }

    pub fn new_req_context(req: &Request<'_>) -> Self {
        let request =
            Arc::new(unsafe { std::mem::transmute::<&Request<'_>, &'static Request<'_>>(req) });

        Self::ActionApplyReq {
            request,
            ops: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn new_resp_context(resp: &Response<'_>) -> Self {
        let response =
            Arc::new(unsafe { std::mem::transmute::<&Response<'_>, &'static Response<'_>>(resp) });

        Self::ActionApplyResp {
            response,
            ops: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
