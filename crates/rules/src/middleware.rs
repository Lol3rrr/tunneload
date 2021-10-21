use crate::Action;

use general::Name;
use general_traits::{ConfigItem, DefaultConfig};

use serde::Serialize;
use stream_httparse::{Request, Response};

/// A Middleware modifies a Request or Response using the
/// provided Action
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Middleware {
    name: Name,
    action: Action,
}

impl Middleware {
    /// Creates a new Middleware from the given
    /// Parameters
    pub fn new(name: Name, action: Action) -> Self {
        Self { name, action }
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
    pub fn get_name(&self) -> &Name {
        &self.name
    }

    /// Returns the Action assosicated with the Middleware
    pub fn get_action(&self) -> &Action {
        &self.action
    }
}

impl ConfigItem for Middleware {
    fn name(&self) -> &Name {
        &self.name
    }
}
impl DefaultConfig for Middleware {
    fn default_name(name: Name) -> Self {
        Self {
            name,
            action: Action::Noop,
        }
    }
}
