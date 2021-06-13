use std::sync::Arc;

use stream_httparse::{streaming_parser::RespParser, Request};

use crate::{
    acceptors::traits::Sender,
    configurator::ConfigItem,
    forwarder::{Forwarder, ServiceConnection},
    internal_services::Internals,
    rules::Rule,
};

use super::{error_messages, HANDLE_TIME_VEC, SERVICE_REQ_VEC, STATUS_CODES_VEC};

mod chunks;
mod response;

pub struct Context<'send, 'forward, S, F> {
    pub sender: &'send mut S,
    pub forwarder: &'forward F,
    pub internals: Arc<Internals>,
}

pub async fn handle<S, F>(
    id: u32,
    request: Request<'_>,
    matched: Arc<Rule>,
    resp_parser: &mut RespParser,
    resp_buf: &mut [u8],
    ctx: Context<'_, '_, S, F>,
) -> Result<(), ()>
where
    S: Sender + Send,
    F: Forwarder + Send,
{
    let middlewares = matched.get_middleware_list();

    // Some metrics related stuff
    let rule_name = matched.name();
    let handle_timer = HANDLE_TIME_VEC
        .get_metric_with_label_values(&[rule_name])
        .unwrap()
        .start_timer();
    SERVICE_REQ_VEC
        .get_metric_with_label_values(&[rule_name])
        .unwrap()
        .inc();

    let mut out_req = request;
    // If a middleware decided that this request should not be processed
    // anymore and instead a certain Response needs to be send to the
    // Client first, sends the given Response to the client and moves
    // on from this request
    if let Err(mid_resp) = middlewares.apply_middlewares_req(&mut out_req) {
        ctx.sender.send_response(&mid_resp).await;

        handle_timer.observe_duration();

        return Ok(());
    }

    let service = matched.service();
    if service.is_internal() {
        let result = ctx.internals.handle(&out_req, matched, ctx.sender);
        return result.await;
    }

    let mut connection = match ctx.forwarder.create_con(&matched).await {
        Some(c) => c,
        None => {
            log::error!("[{}] Connecting to Service", id);
            error_messages::service_unavailable(ctx.sender).await;
            return Err(());
        }
    };

    if let Err(e) = connection.write_req(&out_req).await {
        log::error!("[{}] Sending Request to Backend-Service: {}", id, e);
        error_messages::internal_server_error(ctx.sender).await;
        return Err(());
    }

    let (mut response, left_over_buffer) =
        match response::receive(id, resp_parser, &mut connection, resp_buf).await {
            Some(resp) => resp,
            None => {
                error_messages::internal_server_error(ctx.sender).await;
                return Err(());
            }
        };

    middlewares.apply_middlewares_resp(&out_req, &mut response);

    let (resp_header, resp_body) = response.serialize();
    ctx.sender.send(&resp_header).await;

    // First assumes that it is NOT chunked and should
    // just send the data normally
    if !response.is_chunked() {
        ctx.sender.send(resp_body).await;
    } else {
        chunks::forward(id, &mut connection, ctx.sender, resp_buf, left_over_buffer).await;
    }

    handle_timer.observe_duration();

    STATUS_CODES_VEC
        .get_metric_with_label_values(&[rule_name, response.status_code().serialize()])
        .unwrap()
        .inc();

    Ok(())
}
