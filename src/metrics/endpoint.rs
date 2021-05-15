use stream_httparse::{streaming_parser::ReqParser, Headers, Response, StatusCode};

use prometheus::{Encoder, Registry, TextEncoder};
use tokio::io::AsyncWriteExt;

use log::error;

/// A single HTTP-Endpoint to expose the Metrics
/// for collection
pub struct Endpoint {
    registry: Registry,
}

impl Endpoint {
    /// Creates a new Endpoint using the given Registry
    pub fn new(reg: Registry) -> Self {
        Self { registry: reg }
    }

    async fn handle(mut con: tokio::net::TcpStream, registry: Registry) {
        match con.readable().await {
            Ok(_) => {}
            Err(e) => {
                error!("Checking if the Connection is readable: {}", e);
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
                    error!("Reading from Connection: {}", e);
                    return;
                }
            };
        }
        let request = match parser.finish() {
            Ok(req) => req,
            Err(e) => {
                error!("Could not parse HTTP-Request: {}", e);
                return;
            }
        };

        if request.path() != "/metrics" {
            return;
        }

        let mut buffer = Vec::<u8>::new();
        let encoder = TextEncoder::new();
        encoder.encode(&registry.gather(), &mut buffer).unwrap();

        let mut headers = Headers::new();
        headers.set("Content-Length", buffer.len());
        let resp = Response::new(request.protocol(), StatusCode::OK, headers, buffer);
        let (h_data, data) = resp.serialize();

        match con.write_all(&h_data).await {
            Ok(_) => {}
            Err(e) => {
                error!("Sending Response: {}", e);
                return;
            }
        };
        match con.write_all(data).await {
            Ok(_) => {}
            Err(e) => {
                error!("Sending Response: {}", e);
                return;
            }
        };
    }

    /// Starts the Endpoint on the given Port and will then
    /// serve the Metrics on that Port via HTTP
    pub async fn start(self, port: u32) {
        let listen_addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&listen_addr).await.unwrap();

        loop {
            let con = match listener.accept().await {
                Ok((s, _)) => s,
                Err(e) => {
                    error!("Accepting Connection: {}", e);
                    continue;
                }
            };

            tokio::task::spawn(Self::handle(con, self.registry.clone()));
        }
    }
}
