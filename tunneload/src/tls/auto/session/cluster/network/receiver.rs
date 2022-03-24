use std::sync::Arc;

use async_raft::raft::{AppendEntriesRequest, InstallSnapshotRequest, VoteRequest};
use async_trait::async_trait;
use lazy_static::lazy_static;
use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::{
    tls::auto::{
        session::cluster::{Cluster, ClusterRequest},
        AutoDiscover,
    },
    util::webserver::{Webserver, WebserverHandler},
};
use rules::Matcher;

pub struct Receiver {
    port: u16,
}

impl Receiver {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn start<D>(&self, cluster: Arc<Cluster<D>>)
    where
        D: AutoDiscover + Send + Sync + 'static,
    {
        let handler = Arc::new(ReceiverHandler::new(cluster));

        let bind_addr = format!("0.0.0.0:{}", self.port);
        let server = Webserver::new(bind_addr, handler);

        tokio::task::spawn(server.start());
    }
}

lazy_static! {
    static ref LEADER: Matcher = Matcher::PathPrefix("/leader".to_string());
    static ref LEADER_WRITE: Matcher = Matcher::PathPrefix("/leader/write".to_string());
    static ref ENTRIES: Matcher = Matcher::PathPrefix("/entries".to_string());
    static ref SNAPSHOT: Matcher = Matcher::PathPrefix("/snapshot".to_string());
    static ref VOTE: Matcher = Matcher::PathPrefix("/vote".to_string());
}

struct ReceiverHandler<D> {
    cluster: Arc<Cluster<D>>,
}

impl<D> ReceiverHandler<D>
where
    D: AutoDiscover + Send + Sync + 'static,
{
    pub fn new(cluster: Arc<Cluster<D>>) -> Self {
        Self { cluster }
    }

    async fn handle_rpc(&self, request: &Request<'_>) -> Option<Vec<u8>> {
        if ENTRIES.matches(request) {
            let rpc: AppendEntriesRequest<ClusterRequest> =
                serde_json::from_slice(request.body()).ok()?;
            let result = self.cluster.rpc_append_entries(rpc).await.ok()?;

            return serde_json::to_vec(&result).ok();
        }
        if SNAPSHOT.matches(request) {
            let rpc: InstallSnapshotRequest = serde_json::from_slice(request.body()).ok()?;
            let result = self.cluster.rpc_install_snapshot(rpc).await.ok()?;

            return serde_json::to_vec(&result).ok();
        }
        if VOTE.matches(request) {
            let rpc: VoteRequest = serde_json::from_slice(request.body()).ok()?;
            let result = self.cluster.rpc_vote(rpc).await.ok()?;

            return serde_json::to_vec(&result).ok();
        }
        None
    }

    async fn handle_leader(&self, request: &Request<'_>) -> Option<Vec<u8>> {
        if !self.cluster.is_leader().await {
            tracing::error!("Received Write Request as Leader, but node is not the Leader");
            return None;
        }

        if LEADER_WRITE.matches(request) {
            let req: ClusterRequest = serde_json::from_slice(request.body()).ok()?;
            return match self.cluster.write(req.domain, req.action).await {
                Ok(response) => serde_json::to_vec(&response).ok(),
                Err(_) => None,
            };
        }

        None
    }
}

#[async_trait]
impl<D> WebserverHandler for ReceiverHandler<D>
where
    D: AutoDiscover + Send + Sync + 'static,
{
    async fn handle_request<'req, 'resp>(
        &self,
        request: Request<'req>,
    ) -> Result<Response<'resp>, ()>
    where
        'req: 'resp,
    {
        let result = if LEADER.matches(&request) {
            self.handle_leader(&request).await
        } else {
            self.handle_rpc(&request).await
        };

        match result {
            Some(body) => {
                let mut headers = Headers::new();
                headers.append("Content-Type", "application/json");
                headers.append("Content-Length", body.len());
                let response = Response::new("HTTP/1.1", StatusCode::OK, headers, body);
                Ok(response)
            }
            None => {
                tracing::error!(
                    "Received Request for unknown Raft-Route: {:?}",
                    request.path()
                );
                Err(())
            }
        }
    }
}
