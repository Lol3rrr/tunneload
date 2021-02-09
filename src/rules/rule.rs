use crate::http::Request;
use crate::rules::{Matcher, Middleware, Service};

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    priority: u32,
    matcher: Matcher,
    middlewares: Vec<Middleware>,
    service: Service,
}

impl Rule {
    pub fn new(
        priority: u32,
        matcher: Matcher,
        middlewares: Vec<Middleware>,
        service: Service,
    ) -> Self {
        Self {
            priority,
            matcher,
            middlewares,
            service,
        }
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn matches(&self, req: &Request) -> Option<Service> {
        if self.matcher.matches(req) {
            Some(self.service.clone())
        } else {
            None
        }
    }
}
