use std::sync::Arc;

use async_trait::async_trait;
use stream_httparse::{streaming_parser::ReqParser, Headers, Request, Response, StatusCode};
use tokio::net::TcpListener;

use crate::acceptors::traits::{Receiver, Sender};

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
        C: Receiver + Sender + Send + Sync + 'static,
    {
        let mut parser = ReqParser::new_capacity(2048);
        let mut read_buf = [0; 2048];
        loop {
            loop {
                match Receiver::read(&mut con, &mut read_buf).await {
                    Ok(n) if n == 0 => {
                        break;
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
                        log::error!("Reading from Connection: {}", e);
                        return;
                    }
                };
            }
            let request = match parser.finish() {
                Ok(req) => req,
                Err(e) => {
                    let content = parser.buffer();
                    log::error!("Could not parse HTTP-Request: {}", e);
                    log::error!("{:?}", String::from_utf8(content.to_owned()));
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

            // This is just here for testing purposes
            return;
        }
    }

    /// Starts the Server as a blocking Task
    pub async fn start(self) {
        let listener = TcpListener::bind(&self.bind_addr).await.unwrap();

        loop {
            let (con, _) = listener.accept().await.unwrap();

            tokio::task::spawn(Self::handle(con, self.handler.clone()));
        }
    }
}
