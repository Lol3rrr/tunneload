use std::sync::atomic;

use async_trait::async_trait;
use stream_httparse::{Request, Response};

use super::WebserverHandler;

pub struct MockHandler<'a> {
    counter: atomic::AtomicU64,
    response: Result<Response<'a>, ()>,
}

impl<'a> MockHandler<'a> {
    pub fn new(response: Result<Response<'a>, ()>) -> Self {
        Self {
            counter: atomic::AtomicU64::new(0),
            response,
        }
    }

    pub fn get_counter(&self) -> u64 {
        self.counter.load(atomic::Ordering::SeqCst)
    }
}

#[async_trait]
impl WebserverHandler for MockHandler<'_> {
    async fn handle_request<'req, 'resp>(
        &self,
        _request: Request<'req>,
    ) -> Result<Response<'resp>, ()>
    where
        'req: 'resp,
    {
        self.counter.fetch_add(1, atomic::Ordering::SeqCst);

        match self.response.as_ref() {
            Ok(v) => Ok(v.to_owned()),
            Err(_) => Err(()),
        }
    }
}
