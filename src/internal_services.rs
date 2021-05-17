use std::sync::Arc;

use stream_httparse::Request;

use crate::configurator::ConfigItem;
use crate::{acceptors::traits::Sender, rules::Rule};

mod dashboard;

/// # Returns
/// * Ok: The Connection can still be kept open
/// * Err: The Connection should be closed
pub async fn handle<S>(request: &Request<'_>, rule: Arc<Rule>, sender: &mut S) -> Result<(), ()>
where
    S: Sender + Send,
{
    let service = rule.service();
    match service.name() {
        dashboard::SERVICE_NAME => dashboard::handle(request, rule, sender).await,
        _ => Err(()),
    }
}
