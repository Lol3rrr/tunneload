use crate::{general::Shared, rules::Middleware};

use stream_httparse::{Request, Response};

use std::sync::Arc;

/// Stores a list of all Middlewares.
///
/// This Type is simply a convience type as it makes it easier to
/// store and apply all middlewares for a given Rule.
pub struct MiddlewareList {
    middlewares: Vec<Arc<Middleware>>,
}

impl MiddlewareList {
    /// Applies all the middlewares to the Request, until one returns
    /// a Response to be send directly
    ///
    /// # Returns
    /// * Ok: Everything can proceed as normal and all the middlewares were
    /// successfully applied
    /// * Err(response): Some middleware returned early with an Response that
    /// should be returned immediately
    pub fn apply_middlewares_req<'a>(&self, req: &mut Request<'a>) -> Result<(), Response<'a>> {
        for middleware in self.middlewares.iter() {
            middleware.apply_req(req)?;
        }

        Ok(())
    }

    /// Applies all the middlewares to the provided Response
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
