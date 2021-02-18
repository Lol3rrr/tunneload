use crate::acceptors::webserver::Sender;
use crate::handler::traits::Handler;
use crate::http::streaming_parser::ReqParser;

use log::error;

pub struct Server {
    port: u32,
}

impl Server {
    /// Creates a new Server instance that is ready to start on
    /// the given Port
    pub fn new(port: u32) -> Self {
        Self { port }
    }

    /// Reads and parses the Request for a single connection, then
    /// passes that request onto the given handler
    async fn handle_con<T>(con: tokio::net::TcpStream, handler: T)
    where
        T: Handler + Send + Sync + 'static,
    {
        let mut parser = ReqParser::new_capacity(2048);
        loop {
            match con.readable().await {
                Ok(_) => {}
                Err(e) => {
                    error!("Checking if the Connection is readable: {}", e);
                    return;
                }
            };

            let mut read_buf = [0; 2048];
            match con.try_read(&mut read_buf) {
                Ok(n) if n == 0 => {
                    break;
                }
                Ok(n) => {
                    parser.block_parse(&read_buf[..n]);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    error!("Reading from Connection: {}", e);
                    return;
                }
            };
        }
        let request = match parser.finish() {
            Some(req) => req,
            None => {
                error!("Could not parse HTTP-Request");
                return;
            }
        };

        let sender = Sender::new(con);
        handler.handle(0, request, sender).await;
    }

    /// Actually starts the Webserver and listens for requests,
    /// this function is never expected to actually return and therefore
    /// run for the entire lifetime of the Program
    pub async fn start<T>(self, handler: T)
    where
        T: Handler + Send + Sync + Clone + 'static,
    {
        let listen_addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&listen_addr).await.unwrap();

        loop {
            let con = match listener.accept().await {
                Ok((s, _)) => s,
                Err(e) => {
                    error!("Accepting Connection: {}", e);
                    continue;
                }
            };

            tokio::task::spawn(Self::handle_con(con, handler.clone()));
        }
    }
}
