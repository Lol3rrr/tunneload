#![warn(missing_docs)]
//! Handles all the Rule related stuff

mod manager;
pub use manager::{new, ReadManager};

mod matcher;
pub use matcher::Matcher;

mod service;
pub use service::{ConnectError, Service};

mod action;
pub use action::{Action, CorsOpts};

mod middleware;
pub use middleware::Middleware;

mod middleware_list;
pub use middleware_list::MiddlewareList;

mod rule;
pub use rule::{Rule, RuleTLS};

pub mod parser;
pub mod rule_list;
