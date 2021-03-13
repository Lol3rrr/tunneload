use crate::{
    general::Shared,
    http::{Request, Response},
    rules::Middleware,
};

use std::sync::Arc;

pub struct MiddlewareList {
    middlewares: Vec<Arc<Middleware>>,
}

impl MiddlewareList {
    /// Returns Some(Response) if one of the middlewares
    /// needs to send a Response to the client
    /// and should stop processing the current
    /// Request
    pub fn apply_middlewares_req<'a>(&self, req: &mut Request<'a>) -> Option<Response<'a>> {
        for middleware in self.middlewares.iter() {
            if let Some(r) = middleware.apply_req(req) {
                return Some(r);
            }
        }

        None
    }
    pub fn apply_middlewares_resp<'a, 'b, 'c>(
        &'a self,
        req: &Request<'_>,
        resp: &'b mut Response<'c>,
    ) where
        'a: 'b,
        'a: 'c,
        'c: 'b,
    {
        for middleware in self.middlewares.iter() {
            middleware.apply_resp(req, resp);
        }
    }
}

impl From<&[Shared<Middleware>]> for MiddlewareList {
    fn from(raw_middlewares: &[Shared<Middleware>]) -> Self {
        let count = raw_middlewares.len();
        let mut n_middlewares = Vec::with_capacity(count);

        for tmp in raw_middlewares {
            n_middlewares.push(tmp.get());
        }

        Self {
            middlewares: n_middlewares,
        }
    }
}
