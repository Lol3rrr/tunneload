use crate::http::Request;
use crate::rules::{Matcher, Service};

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    priority: u32,
    matcher: Matcher,
    service: Service,
}

impl Rule {
    pub fn new(priority: u32, matcher: Matcher, service: Service) -> Self {
        Self {
            priority,
            matcher,
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
