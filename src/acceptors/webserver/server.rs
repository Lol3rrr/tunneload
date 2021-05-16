use crate::acceptors::webserver::{Receiver, Sender};
use crate::handler::traits::Handler;
use crate::tls;

use lazy_static::lazy_static;
use prometheus::Registry;

use log::error;

lazy_static! {
    static ref TOTAL_REQS: prometheus::IntCounter = prometheus::IntCounter::new("web_req_total", "The total Number of requests received by the Webserver-Acceptor").unwrap();
    static ref PARSE_TIME: prometheus::Histogram = prometheus::Histogram::with_opts(prometheus::HistogramOpts::new("web_req_parsing", "The Time, in seconds, it takes for a request to be fully received and parsed by the Webserver-Acceptor")).unwrap();
}

/// The actual Webserver that will accept Connections
/// on a single given Port
pub struct Server {
    port: u32,
    tls_conf: Option<tls::ConfigManager>,
}

impl Server {
    /// Creates a new Server instance that is ready to start on
    /// the given Port
    pub fn new(port: u32, reg: Registry, tls_conf: Option<tls::ConfigManager>) -> Self {
        reg.register(Box::new(TOTAL_REQS.clone())).unwrap();
        reg.register(Box::new(PARSE_TIME.clone())).unwrap();

        Self { port, tls_conf }
    }

    /// Reads and parses the Request for a single connection, then
    /// passes that request onto the given handler
    async fn handle_con<T>(
        mut con: tokio::net::TcpStream,
        handler: T,
        tls_conf: Option<tls::ConfigManager>,
    ) where
        T: Handler + Send + Sync + 'static,
    {
        TOTAL_REQS.inc();

        let (read, write) = con.split();

        let mut receiver = Receiver::new(read);
        let mut sender = Sender::new(write);

        match tls_conf {
            Some(tls_config) => {
                let config = tls_config.get_config();
                let session = rustls::ServerSession::new(&config);

                let (mut tls_receiver, mut tls_sender) =
                    match tls::create_sender_receiver(&mut receiver, &mut sender, session).await {
                        Some(s) => s,
                        None => {
                            error!("Could not obtain TLS-Session");
                            return;
                        }
                    };

                handler.handle(0, &mut tls_receiver, &mut tls_sender).await;
            }
            None => {
                handler.handle(0, &mut receiver, &mut sender).await;
            }
        }
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

            tokio::task::spawn(Self::handle_con(
                con,
                handler.clone(),
                self.tls_conf.clone(),
            ));
        }
    }
}