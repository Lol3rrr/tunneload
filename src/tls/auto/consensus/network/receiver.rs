use async_trait::async_trait;
use lazy_static::lazy_static;
use stream_httparse::{Request, Response};

use crate::{rules::Matcher, util::webserver::WebserverHandler};

lazy_static! {
    static ref ENTRIES: Matcher = Matcher::PathPrefix("/entries".to_string());
    static ref SNAPSHOT: Matcher = Matcher::PathPrefix("/snapshot".to_string());
    static ref VOTE: Matcher = Matcher::PathPrefix("/vote".to_string());
}

pub struct NetworkReceiver {}

impl NetworkReceiver {
    pub fn new(port: usize) -> Self {
        Self {}
    }

    /// This function listens for RPC calls
    ///
    /// Needed Endpoints:
    /// * `/entries/append`
    /// * `/snapshot/install`
    /// * `/vote`
    pub async fn run() {}
}

#[async_trait]
impl WebserverHandler for NetworkReceiver {
    async fn handle_request<'req, 'resp>(
        &self,
        request: Request<'req>,
    ) -> Result<Response<'resp>, ()>
    where
        'req: 'resp,
    {
        if ENTRIES.matches(&request) {
            todo!("Implement '/entries/append'");
        }
        if SNAPSHOT.matches(&request) {
            todo!("Implement '/snapshot/install'");
        }
        if VOTE.matches(&request) {
            todo!("Implement '/vote'");
        }

        log::error!(
            "Received Request for unknown Raft-Route: {:?}",
            request.path()
        );
        Err(())
    }
}
