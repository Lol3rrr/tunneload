use std::sync::Arc;

use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::{acceptors::traits::Sender, rules::Rule};

pub const SERVICE_NAME: &'static str = "dashboard@internal";

pub async fn handle<S>(request: &Request<'_>, rule: Arc<Rule>, sender: &mut S) -> Result<(), ()>
where
    S: Sender + Send,
{
    log::info!("Received Dashboard Request");
    log::info!("Request: {:?}", request);

    let response = Response::new("HTTP/1.1", StatusCode::OK, Headers::new(), vec![]);

    let (response_head, response_body) = response.serialize();
    let head_length = response_head.len();
    sender.send(response_head, head_length).await;
    let body_length = response_body.len();
    sender.send(response_body.to_vec(), body_length).await;

    Ok(())
}
