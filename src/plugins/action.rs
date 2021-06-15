use std::{convert::TryInto, sync::Arc};

use serde::ser::SerializeMap;
use stream_httparse::{streaming_parser::RespParser, Headers, Request, Response, StatusCode};
use wasmer::{Module, Store};

#[derive(Debug)]
pub enum MiddlewareOp {
    SetPath(String),
    SetHeader(String, String),
    SetBody(Vec<u8>),
}

use crate::plugins::start_instance;

use super::{
    api::{self, PluginEnv},
    InstantiatePlugin,
};

/// An actual Instance of a Plugin, which is additionally contains
/// a Configuration if so desired
#[derive(Debug, Clone)]
pub struct ActionPluginInstance {
    name: String,
    store: Store,
    module: Module,
    config: Arc<Vec<u8>>,
    config_size: i32,
}

impl PartialEq for ActionPluginInstance {
    fn eq(&self, other: &ActionPluginInstance) -> bool {
        self.name.eq(&other.name)
    }
}

impl serde::Serialize for ActionPluginInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let map = serializer.serialize_map(Some(0))?;
        map.end()
    }
}

impl ActionPluginInstance {
    /// This applies the loaded WASM module to the given Request
    pub fn apply_req<'owned>(&self, req: &mut Request) -> Result<(), Response<'owned>> {
        let exec_env = PluginEnv::new(
            self.config.clone(),
            api::PluginContext::new_req_context(req),
        );

        let instance = match start_instance(&self.store, &exec_env, &self.module) {
            Some(i) => i,
            None => return Ok(()),
        };

        let instance_apply_req = match instance
            .exports
            .get_native_function::<(i32, i32, i32, i32), i32>("apply_req")
        {
            Ok(f) => f,
            Err(_) => {
                return Ok(());
            }
        };

        let config_size = self.config_size;
        let path_length = req.path().len() as i32;
        let body_size = req.body().len() as i32;
        let max_header_size = req.headers().get_max_value_size() as i32;

        let return_value =
            match instance_apply_req.call(config_size, path_length, body_size, max_header_size) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("Executing Plugin: {:?}", e);
                    return Err(Response::new(
                        "HTTP/1.1",
                        StatusCode::InternalServerError,
                        Headers::new(),
                        Vec::new(),
                    ));
                }
            };

        let ops = match &exec_env.context {
            api::PluginContext::ActionApplyReq { ops, .. } => ops,
            _ => unreachable!("The Context should always be ActionApplyReq"),
        };
        let mut ops = ops.lock().unwrap();
        let drain_iter = ops.drain(..);
        for op in drain_iter {
            match op {
                MiddlewareOp::SetPath(path) => {
                    req.set_path_owned(path);
                }
                MiddlewareOp::SetHeader(key, value) => {
                    req.header_mut().set(key, value);
                }
                MiddlewareOp::SetBody(data) => {
                    req.set_body(data);
                }
            }
        }

        match return_value {
            -1 => Ok(()),
            _ if return_value > 0 => {
                let return_value = return_value as usize;
                let resp_size_bytes = exec_env.get_memory_slice(return_value, 4);
                let resp_size = u32::from_be_bytes(resp_size_bytes.as_slice().try_into().unwrap());

                let raw_resp_bytes =
                    exec_env.get_memory_slice(return_value + 4, resp_size as usize);
                let mut parser = RespParser::new_capacity(0);
                let (done, _) = parser.block_parse(raw_resp_bytes.as_slice());
                if !done {
                    panic!("Returned Response could not be parsed again");
                }

                let resp = parser.finish_owned().unwrap();

                Err(resp)
            }
            _ => {
                log::error!("Unexpected Return-Value: {}", return_value);
                Ok(())
            }
        }
    }

    /// This applies the loaded WASM module to the given Request
    pub fn apply_resp(&self, resp: &mut Response) {
        let exec_env = PluginEnv::new(
            self.config.clone(),
            api::PluginContext::new_resp_context(resp),
        );

        let instance = match start_instance(&self.store, &exec_env, &self.module) {
            Some(i) => i,
            None => return,
        };

        let instance_apply_resp = match instance
            .exports
            .get_native_function::<(i32, i32, i32), i32>("apply_resp")
        {
            Ok(f) => f,
            Err(_) => return,
        };

        let config_size = self.config_size;
        let body_size = resp.body().len() as i32;
        let max_header_size = resp.headers().get_max_value_size() as i32;

        if let Err(e) = instance_apply_resp.call(config_size, body_size, max_header_size) {
            log::error!("Executing Plugin: {:?}", e);
            return;
        }

        let ops = match exec_env.context {
            api::PluginContext::ActionApplyResp { ops, .. } => ops,
            _ => unreachable!("The Context should always be ActionApplyResp"),
        };
        let mut ops = ops.lock().unwrap();
        let drain_iter = ops.drain(..);
        for op in drain_iter {
            match op {
                MiddlewareOp::SetPath(_) => {}
                MiddlewareOp::SetHeader(key, value) => {
                    resp.add_header(key, value);
                }
                MiddlewareOp::SetBody(data) => {
                    resp.set_body(data);
                }
            }
        }
    }
}

impl InstantiatePlugin for ActionPluginInstance {
    fn instantiate(name: String, store: Store, module: Module, config: Option<Vec<u8>>) -> Self {
        let (config_size, config) = match config {
            Some(config) => (config.len() as i32, Arc::new(config)),
            None => (-1, Arc::new(Vec::new())),
        };

        Self {
            name,
            store,
            module,
            config,
            config_size,
        }
    }
}
