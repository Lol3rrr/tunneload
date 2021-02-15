use crate::http::{Request, Response};
use crate::rules::Action;

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

    pub fn apply_req(&self, req: &mut Request) {
        self.action.apply_req(req)
    }
    pub fn apply_resp(&self, resp: &mut Response) {
        self.action.apply_resp(resp)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}
