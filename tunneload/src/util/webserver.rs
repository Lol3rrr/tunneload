use std::sync::Arc;

use async_trait::async_trait;
use stream_httparse::{streaming_parser::ReqParser, Headers, Request, Response, StatusCode};
use tokio::net::TcpListener;

use general_traits::{Receiver, Sender};

#[cfg(test)]
mod mocks;

/// This trait provides the general Interface the Webserver accepts its
/// Handler to implement
#[async_trait]
pub trait WebserverHandler {
    /// This function is called for every received Request.
    ///
    /// # Returns:
    /// * Ok(response): The Handler executed as planned and the given response should
    /// be send back to the user
    /// * Err(_): Some unrecoverable error occured in the Handler. The Server will then
    /// send an InternalServerError as response
    async fn handle_request<'req, 'resp>(
        &self,
        request: Request<'req>,
    ) -> Result<Response<'resp>, ()>
    where
        'req: 'resp;
}

/// A General Webserver that listens on a given Port and uses
/// the given Handler to actually handle all of its requests.
///
/// This is nowhere near as performant and supported/build out
/// as the normal Handler, etc. as this is only intended for things
/// like Metrics, inter-Tunneload comms and the like.
pub struct Webserver<H> {
    bind_addr: String,
    handler: Arc<H>,
}

impl<H> Webserver<H>
where
    H: WebserverHandler + Send + Sync + 'static,
{
    /// Creates a new Webserver from the given Configuration
    pub fn new(bind_addr: String, handler: Arc<H>) -> Self {
        Self { bind_addr, handler }
    }

    async fn handle<C>(mut con: C, handler: Arc<H>)
    where
        C: Receiver + Sender + Send + Sync,
    {
        let mut parser = ReqParser::new_capacity(2048);
        let mut read_buf = [0; 2048];
        loop {
            loop {
                match Receiver::read(&mut con, &mut read_buf).await {
                    Ok(n) if n == 0 => {
                        // Received EOF meaning that there will be no more data coming for this
                        return;
                    }
                    Ok(n) => {
                        let (done, _) = parser.block_parse(&read_buf[..n]);
                        if done {
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        tracing::error!("Reading from Connection: {}", e);
                        return;
                    }
                };
            }
            let request = match parser.finish() {
                Ok(req) => req,
                Err(e) => {
                    let content = parser.buffer();
                    tracing::error!("Could not parse HTTP-Request: {}", e);
                    tracing::error!("{:?}", String::from_utf8(content.to_owned()));
                    return;
                }
            };

            let resp = match handler.handle_request(request).await {
                Ok(r) => r,
                Err(_) => Response::new(
                    "HTTP/1.1",
                    StatusCode::InternalServerError,
                    Headers::new(),
                    Vec::new(),
                ),
            };

            con.send_response(&resp).await;

            parser.clear();
        }
    }

    /// Starts the Server as a blocking Task
    pub async fn start(self) {
        let listener = TcpListener::bind(&self.bind_addr).await.unwrap();

        loop {
            let (con, _) = match listener.accept().await {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Accepting Client: {:?}", e);
                    continue;
                }
            };

            tokio::task::spawn(Self::handle(con, self.handler.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use stream_httparse::Method;

    use crate::acceptors;

    use super::*;

    #[tokio::test]
    async fn single_request() {
        let sample_request = Request::new("HTTP/1.1", Method::GET, "/test/", Headers::new(), &[]);

        let handler = Arc::new(mocks::MockHandler::new(Err(())));

        let (head, body) = sample_request.serialize();
        let mut receiver = acceptors::mocks::Receiver::new();
        receiver.add_chunk(head);
        receiver.add_chunk(body.to_vec());

        let mut sender = acceptors::mocks::Sender::new();
        let connection = acceptors::mocks::Connection::new(&mut receiver, &mut sender);

        Webserver::handle(connection, handler.clone()).await;

        assert_eq!(1, handler.get_counter());
    }

    #[tokio::test]
    async fn two_requests() {
        let sample_request = Request::new("HTTP/1.1", Method::GET, "/test/", Headers::new(), &[]);

        let handler = Arc::new(mocks::MockHandler::new(Ok(Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            Headers::new(),
            Vec::new(),
        ))));

        let (head, _body) = sample_request.serialize();
        let mut receiver = acceptors::mocks::Receiver::new();
        receiver.add_chunk(head.clone());
        // Do not send the empty Body as it would cause a zero sized read
        // receiver.add_chunk(body.to_vec());
        receiver.add_chunk(head);
        // Do not send the empty Body as it would cause a zero sized read
        // receiver.add_chunk(body.to_vec());

        let mut sender = acceptors::mocks::Sender::new();
        let connection = acceptors::mocks::Connection::new(&mut receiver, &mut sender);

        Webserver::handle(connection, handler.clone()).await;

        assert_eq!(2, handler.get_counter());
    }
}
