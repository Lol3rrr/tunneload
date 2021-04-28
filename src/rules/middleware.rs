use crate::configurator::{ConfigItem, DefaultConfig};
use crate::rules::Action;

use stream_httparse::{Request, Response};

/// A Middleware modifies a Request or Response using the
/// provided Action
#[derive(Clone, Debug, PartialEq)]
pub struct Middleware {
    name: String,
    action: Action,
}

impl Middleware {
    pub fn new(name: &str, action: Action) -> Self {
        Self {
            name: name.to_owned(),
            action,
        }
    }

    pub fn apply_req<'a>(&self, req: &mut Request<'a>) -> Option<Response<'a>> {
        self.action.apply_req(req)
    }
    pub fn apply_resp<'a, 'b, 'c>(&'a self, req: &Request<'_>, resp: &'b mut Response<'c>)
    where
        'a: 'b,
        'a: 'c,
        'c: 'b,
    {
        self.action.apply_resp(req, resp)
    }

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
