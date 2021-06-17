//! All the Handshake related stuff for Websockets

use stream_httparse::{streaming_parser::RespParser, Request, Response};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

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
                tracing::error!("Reading from Connection: {:?}", e);
                return None;
            }
        };
    }

    match parser.finish() {
        Ok(r) => Some(r),
        Err(e) => {
            tracing::error!("Finish Parsing Response: {:?}", e);
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
) -> Option<(OwnedReadHalf, OwnedWriteHalf)>
where
    S: Sender + Send,
{
    let _initial_data = match client_initial::parse(initial) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Parsing initial Handshake: {:?}", e);
            return None;
        }
    };

    // TODO
    // Actually "handle" the initial Data and possibly check
    // against certain filters, etc.

    let mut connection = match rule.service().connect().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Connecting to Service: {:?}", e);
            return None;
        }
    };

    // Actually sends out the initial Websocket Message to the
    // target backend Service
    let (head, body) = initial.serialize();
    if let Err(e) = connection.write_all(&head).await {
        tracing::error!("Forwarding initial HTTP-Head: {:?}", e);
        return None;
    }
    if let Err(e) = connection.write_all(&body).await {
        tracing::error!("Forwarding initial HTTP-Head: {:?}", e);
        return None;
    }

    let response = match receive_response(&mut connection, resp_parser).await {
        Some(r) => r,
        None => {
            return None;
        }
    };

    // TODO
    // Actually handle the Response and determine if it worked correctly and stuff

    sender.send_response(&response).await;

    Some(connection.into_split())
}
