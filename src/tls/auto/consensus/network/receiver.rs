use async_raft::raft::{AppendEntriesRequest, InstallSnapshotRequest, VoteRequest};
use async_trait::async_trait;
use lazy_static::lazy_static;
use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::{rules::Matcher, tls::auto::consensus, util::webserver::WebserverHandler};

lazy_static! {
    static ref ENTRIES: Matcher = Matcher::PathPrefix("/entries".to_string());
    static ref SNAPSHOT: Matcher = Matcher::PathPrefix("/snapshot".to_string());
    static ref VOTE: Matcher = Matcher::PathPrefix("/vote".to_string());
}

pub struct NetworkReceiver {
    raft: async_raft::Raft<
        consensus::Request,
        consensus::Response,
        consensus::Network,
        consensus::Storage,
    >,
}

impl NetworkReceiver {
    pub fn new(
        raft: async_raft::Raft<
            consensus::Request,
            consensus::Response,
            consensus::Network,
            consensus::Storage,
        >,
    ) -> Self {
        Self { raft }
    }

    async fn handle_rpc(&self, request: &Request<'_>) -> Option<Vec<u8>> {
        if ENTRIES.matches(request) {
            let rpc: AppendEntriesRequest<consensus::Request> =
                serde_json::from_slice(request.body()).unwrap();
            let result = self.raft.append_entries(rpc).await.unwrap();

            let body = serde_json::to_vec(&result).unwrap();
            return Some(body);
        }
        if SNAPSHOT.matches(request) {
            let rpc: InstallSnapshotRequest = serde_json::from_slice(request.body()).unwrap();
            let result = self.raft.install_snapshot(rpc).await.unwrap();

            let body = serde_json::to_vec(&result).unwrap();
            return Some(body);
        }
        if VOTE.matches(request) {
            let rpc: VoteRequest = serde_json::from_slice(request.body()).unwrap();
            let result = self.raft.vote(rpc).await.unwrap();

            let body = serde_json::to_vec(&result).unwrap();
            return Some(body);
        }
        None
    }
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
        match self.handle_rpc(&request).await {
            Some(body) => {
                let mut headers = Headers::new();
                headers.append("Content-Type", "application/json");
                headers.append("Content-Length", body.len());
                let response = Response::new("HTTP/1.1", StatusCode::OK, headers, body);
                Ok(response)
            }
            None => {
                log::error!(
                    "Received Request for unknown Raft-Route: {:?}",
                    request.path()
                );
                Err(())
            }
        }
    }
}
