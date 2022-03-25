use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
};

use futures::Future;
use stream_httparse::{Request, Response};
use wasmer::{HostEnvInitError, Instance, Memory, WasmerEnv};

use super::{REQUEST_RESSOURCE_ID, RESPONSE_RESSOURCE_ID};
use crate::{
    acceptor::{
        AcceptorMessage, AcceptorPluginReceiver, AcceptorPluginSender, AcceptorQueueReceiver,
    },
    action::MiddlewareOp,
};

#[derive(Debug)]
pub struct EnvMemory {
    memory: Mutex<Option<Memory>>,
}

impl EnvMemory {
    pub fn new() -> Self {
        Self {
            memory: Mutex::new(None),
        }
    }

    pub fn init(&self, memory: Memory) {
        *self
            .memory
            .lock()
            .expect("Obtaining the Lock should never fail") = Some(memory);
    }

    pub fn get_slice(&self, range: std::ops::Range<usize>) -> Option<MemoryGuard> {
        let raw_guard = self
            .memory
            .lock()
            .expect("Obtaining the Lock should never fail");

        raw_guard
            .as_ref()
            .map(|mem| MemoryGuard::new(mem.clone(), range))
    }
}

pub struct MemoryGuard {
    memory: Memory,
    range: std::ops::Range<usize>,
}

impl MemoryGuard {
    pub fn new(memory: Memory, range: std::ops::Range<usize>) -> Self {
        Self { memory, range }
    }

    pub fn as_slice<'guard, 'slice>(&'guard self) -> &'slice [u8]
    where
        'guard: 'slice,
    {
        let raw = unsafe { self.memory.data_unchecked() };
        &raw[self.range.clone()]
    }
    pub fn as_mut_slice<'guard, 'slice>(&'guard self) -> &'slice mut [u8]
    where
        'guard: 'slice,
    {
        let raw = unsafe { self.memory.data_unchecked_mut() };
        &mut raw[self.range.clone()]
    }
}

pub type StartHandlerFunc = Arc<
    dyn Fn(
            i32,
            AcceptorPluginReceiver,
            AcceptorPluginSender,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[derive(Clone)]
pub struct PluginEnv {
    memory: Arc<EnvMemory>,
    pub config: Arc<Vec<u8>>,
    pub context: PluginContext,
}

#[derive(Clone)]
pub enum PluginContext {
    TypeCheck,
    Config {
        config_str: String,
    },
    ActionApplyReq {
        request: Arc<&'static Request<'static>>,
        ops: Arc<Mutex<Vec<MiddlewareOp>>>,
    },
    ActionApplyResp {
        request: Arc<&'static Request<'static>>,
        response: Arc<&'static Response<'static>>,
        ops: Arc<Mutex<Vec<MiddlewareOp>>>,
    },
    Acceptor {
        start_handler: StartHandlerFunc,
        send_queue_rx: Arc<Mutex<AcceptorQueueReceiver>>,
        send_queue_tx: Arc<Mutex<std::sync::mpsc::Sender<AcceptorMessage>>>,
        to_tl_queues: Arc<Mutex<HashMap<i32, tokio::sync::mpsc::UnboundedSender<Vec<u8>>>>>,
    },
}

impl WasmerEnv for PluginEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        let memory = instance
            .exports
            .get_memory("memory")
            .expect("Memory should always exist in the Instance");
        self.memory.init(memory.clone());
        Ok(())
    }
}

impl PluginEnv {
    pub fn new(config: Arc<Vec<u8>>, context: PluginContext) -> Self {
        Self {
            memory: Arc::new(EnvMemory::new()),
            config,
            context,
        }
    }

    pub fn get_memory_slice(&self, start: usize, size: usize) -> MemoryGuard {
        self.memory
            .get_slice(start..start + size)
            .expect("This should always just work")
    }

    pub fn load_string(&self, target: i32, target_length: i32) -> String {
        let length = target_length as usize;
        let start = target as usize;

        let mem = self.get_memory_slice(start, length);

        let mut bytes: Vec<u8> = Vec::with_capacity(length);
        bytes.extend_from_slice(mem.as_slice());

        String::from_utf8(bytes).expect("All loaded Strings should be valid utf8")
    }

    pub fn set_string(&self, target: i32, data: &str) {
        let raw_data = data.as_bytes();

        let start = target as usize;

        let mem = self.get_memory_slice(start, raw_data.len());
        mem.as_mut_slice().copy_from_slice(raw_data);
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
    pub fn body(&self, ressource: i32) -> Option<&[u8]> {
        match ressource {
            REQUEST_RESSOURCE_ID => match self {
                Self::ActionApplyReq { request, .. } => Some(request.body()),
                Self::ActionApplyResp { request, .. } => Some(request.body()),
                _ => None,
            },
            RESPONSE_RESSOURCE_ID => match self {
                Self::ActionApplyResp { response, .. } => Some(response.body()),
                _ => None,
            },
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
    pub fn new_resp_context(req: &Request<'_>, resp: &Response<'_>) -> Self {
        let request =
            Arc::new(unsafe { std::mem::transmute::<&Request<'_>, &'static Request<'_>>(req) });
        let response =
            Arc::new(unsafe { std::mem::transmute::<&Response<'_>, &'static Response<'_>>(resp) });

        Self::ActionApplyResp {
            request,
            response,
            ops: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn new_acceptor_context<F>(
        recv: AcceptorQueueReceiver,
        send: std::sync::mpsc::Sender<AcceptorMessage>,
        handler: F,
    ) -> Self
    where
        F: Fn(
                i32,
                AcceptorPluginReceiver,
                AcceptorPluginSender,
            ) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
            + Send
            + Sync
            + 'static,
    {
        Self::Acceptor {
            start_handler: Arc::new(handler),
            send_queue_rx: Arc::new(Mutex::new(recv)),
            send_queue_tx: Arc::new(Mutex::new(send)),
            to_tl_queues: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
