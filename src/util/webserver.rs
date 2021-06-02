use std::sync::Arc;

use async_trait::async_trait;
use stream_httparse::{streaming_parser::ReqParser, Headers, Request, Response, StatusCode};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

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

    async fn handle(mut con: TcpStream, handler: Arc<H>) {
        loop {
            match con.readable().await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Checking if the Connection is readable: {}", e);
                    return;
                }
            };

            let mut parser = ReqParser::new_capacity(2048);
            let mut read_buf = [0; 2048];
            loop {
                match con.try_read(&mut read_buf) {
                    Ok(n) if n == 0 => {
                        break;
                    }
                    Ok(n) => {
                        parser.block_parse(&read_buf[..n]);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        break;
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
                    log::error!("Could not parse HTTP-Request: {}", e);
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

            let (h_data, data) = resp.serialize();

            match con.write_all(&h_data).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Sending Response: {}", e);
                    return;
                }
            };
            match con.write_all(data).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Sending Response: {}", e);
                    return;
                }
            };
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
