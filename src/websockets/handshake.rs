//! All the Handshake related stuff for Websockets

use stream_httparse::{streaming_parser::RespParser, Request, Response};

use crate::{acceptors::traits::Sender, forwarder::ServiceConnection, rules::Rule};

mod client_initial;

async fn receive_response<'a, 'b, R>(
    recv: &mut R,
    parser: &'a mut RespParser,
) -> Option<Response<'b>>
where
    'a: 'b,
    R: ServiceConnection + Send,
{
    let mut read_buf = [0; 512];

    loop {
        match recv.read(&mut read_buf).await {
            Ok(n) if n == 0 => {
                return None;
            }
            Ok(n) => {
                let (done, _) = parser.block_parse(&read_buf[0..n]);
                if done {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                log::error!("Reading from Connection: {:?}", e);
                return None;
            }
        };
    }

    match parser.finish() {
        Ok(r) => Some(r),
        Err(e) => {
            log::error!("Finish Parsing Response: {:?}", e);
            None
        }
    }
}

/// Handles the initial Websocket Handshake
pub async fn handle<S>(
    initial: &Request<'_>,
    sender: &mut S,
    rule: &Rule,
    resp_parser: &mut RespParser,
) where
    S: Sender + Send,
{
    let _initial_data = match client_initial::parse(initial) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Parsing initial Handshake: {:?}", e);
            return;
        }
    };

    // TODO
    // Actually "handle" the initial Data and possibly check
    // against certain filters, etc.

    let mut connection = match rule.service().connect().await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Connecting to Service: {:?}", e);
            return;
        }
    };

    // Actually sends out the initial Websocket Message to the
    // target backend Service
    let (head, body) = initial.serialize();
    if let Err(e) = connection.write_all(&head).await {
        log::error!("Forwarding initial HTTP-Head: {:?}", e);
        return;
    }
    if let Err(e) = connection.write_all(&body).await {
        log::error!("Forwarding initial HTTP-Head: {:?}", e);
        return;
    }

    let response = match receive_response(&mut connection, resp_parser).await {
        Some(r) => r,
        None => {
            return;
        }
    };

    // TODO
    // Actually handle the Response and determine if it worked correctly and stuff

    let (resp_header, resp_body) = response.serialize();
    let resp_header_length = resp_header.len();
    sender.send(resp_header, resp_header_length).await;
    let resp_body_length = resp_body.len();
    sender.send(resp_body.to_vec(), resp_body_length).await;
}
