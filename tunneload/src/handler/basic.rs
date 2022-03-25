use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use crate::{forwarder::Forwarder, internal_services::Internals, websockets};
use general_traits::{Handler, Receiver, Sender};
use rules::ReadManager;

use stream_httparse::streaming_parser::{ReqParser, RespParser};

use async_trait::async_trait;

use lazy_static::lazy_static;
use prometheus::Registry;

use tracing::Level;

use self::http_handler::Context;

mod error_messages;
mod request;

mod http_handler;
mod ws_handler;

lazy_static! {
    pub static ref HANDLE_TIME_VEC: prometheus::HistogramVec = prometheus::HistogramVec::new(
        prometheus::HistogramOpts::new(
            "basic_handling",
            "The Time, in seconds, it takes for a request to be fully handled"
        ),
        &["service"]
    )
    .expect("Creating a Metric should never fail");
    pub static ref SERVICE_REQ_VEC: prometheus::IntCounterVec = prometheus::IntCounterVec::new(
        prometheus::Opts::new("service_reqs", "The Requests going to each service"),
        &["service"]
    )
    .expect("Creating a Metric should never fail");
    pub static ref STATUS_CODES_VEC: prometheus::IntCounterVec = prometheus::IntCounterVec::new(
        prometheus::Opts::new("status_codes", "The StatusCodes returned by each service"),
        &["service", "status_code"]
    )
    .expect("Creating a Metric should never fail");
}

/// A Basic Handler that parses the Requests, matches them against
/// all the known Rules, applies all the matching middlewares accordingly
/// and forwards the Request using the provided Forwarder
#[derive(Clone)]
pub struct BasicHandler<F> {
    rules: ReadManager,
    forwarder: F,
    internals: Arc<Internals>,
}

impl<F> Debug for BasicHandler<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BasicHandler ()")
    }
}

impl<F> BasicHandler<F>
where
    F: Forwarder,
{
    /// Creates a new Handler from the provided data
    pub fn new(
        rules_manager: ReadManager,
        forwarder: F,
        internals: Internals,
        registry: Option<Registry>,
    ) -> Self {
        if let Some(reg) = registry {
            if let Err(e) = reg.register(Box::new(HANDLE_TIME_VEC.clone())) {
                tracing::error!("Registering Metric: {:?}", e);
            }
            if let Err(e) = reg.register(Box::new(SERVICE_REQ_VEC.clone())) {
                tracing::error!("Registering Metric: {:?}", e);
            }
            if let Err(e) = reg.register(Box::new(STATUS_CODES_VEC.clone())) {
                tracing::error!("Registering Metric: {:?}", e);
            }
        }

        Self {
            rules: rules_manager,
            forwarder,
            internals: Arc::new(internals),
        }
    }
}

#[async_trait]
impl<F> Handler for BasicHandler<F>
where
    F: Forwarder + Send + Sync,
{
    #[tracing::instrument]
    async fn handle<R, S>(&self, id: u32, mut receiver: R, mut sender: S)
    where
        R: Receiver + 'static,
        S: Sender + 'static,
    {
        let mut keep_alive = true;

        let mut req_buf = [0; 2048];
        let mut req_offset = 0;
        let mut req_parser = ReqParser::new_capacity(2048);

        let mut resp_buf = [0; 2048];
        let mut resp_parser = RespParser::new_capacity(2048);

        while keep_alive {
            let request =
                match request::receive(&mut req_parser, &mut receiver, &mut req_buf, req_offset)
                    .await
                {
                    Ok((r, n_offset)) => {
                        req_offset = n_offset;
                        r
                    }
                    Err(e) => {
                        match e {
                            request::RecvReqError::EOF => {
                                tracing::event!(Level::DEBUG, "Received EOF");
                            }
                            _ => {
                                tracing::event!(Level::ERROR, "Received Invalid Request: {:?}", e);
                            }
                        };
                        error_messages::bad_request(&mut sender).await;
                        return;
                    }
                };
            keep_alive = request.is_keep_alive();

            let matched = match self.rules.match_req(&request) {
                Some(m) => m,
                None => {
                    tracing::event!(Level::ERROR, "No Rule matched the Request: {:?}", request);
                    error_messages::not_found(&mut sender).await;
                    return;
                }
            };

            // Check if the received Request is the starting Handshake of a Websocket connection
            if websockets::is_websocket(&request) {
                ws_handler::handle(id, request, receiver, sender, matched, &mut resp_parser).await;

                return;
            }

            let internals = self.internals.clone();
            if http_handler::handle(
                id,
                request,
                matched,
                &mut resp_parser,
                &mut resp_buf,
                Context {
                    sender: &mut sender,
                    forwarder: &self.forwarder,
                    internals,
                },
            )
            .await
            .is_err()
            {
                return;
            }

            // Clearing the Parser and therefore making it ready
            // parse a new Request without needing to allocate
            // another block
            req_parser.clear();
            resp_parser.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use general::Group;
    use general::Name;
    use general::Shared;
    use rules::{Matcher, Rule, Service};

    use super::*;

    use crate::acceptors::mocks::Receiver as MockReceiver;
    use crate::acceptors::mocks::Sender as MockSender;
    use crate::forwarder::mocks::Forwarder as MockForwarder;
    use crate::forwarder::mocks::ServiceConnection as MockServiceConnection;

    #[tokio::test]
    async fn basic_handle_valid() {
        let mut tmp_service_con = MockServiceConnection::new();
        tmp_service_con.add_chunk("HTTP/1.1 200 OK\r\n\r\n".as_bytes().to_vec());
        let tmp_forwarder = MockForwarder::new(tmp_service_con);

        let mut receiver = MockReceiver::new();
        receiver.add_chunk("GET /api/test/ HTTP/1.1\r\n\r\n".as_bytes().to_vec());
        let sender = MockSender::new();

        let (read, mut write) = rules::new();
        write.set_single(Rule::new(
            Name::new("test-rule", Group::Internal),
            12,
            Matcher::PathPrefix("/api".to_owned()),
            vec![],
            Shared::new(Service::new(
                Name::new("test-service", Group::File {}),
                vec![],
            )),
        ));

        let handler: BasicHandler<MockForwarder> =
            BasicHandler::new(read.clone(), tmp_forwarder, Internals::new(), None);

        handler.handle(12, receiver, sender.clone()).await;

        assert_eq!(
            Ok("HTTP/1.1 200 OK\r\n\r\n".to_owned()),
            String::from_utf8(sender.get_combined_data())
        );
    }

    #[tokio::test]
    async fn basic_handle_invalid_no_rules_match() {
        let mut tmp_service_con = MockServiceConnection::new();
        tmp_service_con.add_chunk("HTTP/1.1 200 OK\r\n\r\n".as_bytes().to_vec());
        let tmp_forwarder = MockForwarder::new(tmp_service_con);

        let mut receiver = MockReceiver::new();
        receiver.add_chunk("GET /test/ HTTP/1.1\r\n\r\n".as_bytes().to_vec());
        let sender = MockSender::new();

        let (read, mut write) = rules::new();
        write.set_single(Rule::new(
            Name::new("test-rule", Group::Internal),
            12,
            Matcher::PathPrefix("/api".to_owned()),
            vec![],
            Shared::new(Service::new(
                Name::new("test-service", Group::File {}),
                vec![],
            )),
        ));

        let handler: BasicHandler<MockForwarder> =
            BasicHandler::new(read.clone(), tmp_forwarder, Internals::new(), None);

        handler.handle(12, receiver, sender.clone()).await;

        assert_eq!(
            Ok("HTTP/1.1 404 Not Found\r\n\r\nNot Found".to_owned()),
            String::from_utf8(sender.get_combined_data())
        );
    }
}
