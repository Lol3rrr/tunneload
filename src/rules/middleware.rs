use crate::configurator::{ConfigItem, DefaultConfig};
use crate::rules::Action;

use serde::Serialize;
use stream_httparse::{Request, Response};

/// A Middleware modifies a Request or Response using the
/// provided Action
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Middleware {
    name: String,
    action: Action,
}

impl Middleware {
    /// Creates a new Middleware from the given
    /// Parameters
    pub fn new(name: &str, action: Action) -> Self {
        Self {
            name: name.to_owned(),
            action,
        }
    }

    /// Applies the Middleware to the given Request
    pub fn apply_req<'a>(&self, req: &mut Request<'a>) -> Result<(), Response<'a>> {
        self.action.apply_req(req)
    }
    /// Applies the Middleware to the given Response
    pub fn apply_resp<'a, 'b, 'c>(&'a self, req: &Request<'_>, resp: &'b mut Response<'c>)
    where
        'a: 'b,
        'a: 'c,
        'c: 'b,
    {
        self.action.apply_resp(req, resp)
    }

    /// Returns the Name of the Middleware
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl ConfigItem for Middleware {
    fn name(&self) -> &str {
        &self.name
    }
}
impl DefaultConfig for Middleware {
    fn default_name(name: String) -> Self {
        Self {
            name,
            action: Action::Noop,
        }
    }
}
