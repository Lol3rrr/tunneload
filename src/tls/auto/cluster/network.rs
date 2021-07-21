use std::sync::Arc;

use async_raft::{
    raft::{
        AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest,
        InstallSnapshotResponse, VoteRequest, VoteResponse,
    },
    NodeId,
};
use async_trait::async_trait;
use lazy_static::lazy_static;
use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::util::webserver::{Webserver, WebserverHandler};
use crate::{
    rules::Matcher,
    tls::auto::{cluster::Cluster, AutoDiscover},
};

use super::ClusterRequest;

mod ids;
pub use ids::*;

pub struct Sender {
    client: reqwest::Client,
}

#[derive(Debug)]
pub enum SendError {
    Reqwest(reqwest::Error),
    ParseURL(url::ParseError),
    ErrorResp,
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SendError {}

impl From<reqwest::Error> for SendError {
    fn from(raw: reqwest::Error) -> Self {
        Self::Reqwest(raw)
    }
}
impl From<url::ParseError> for SendError {
    fn from(raw: url::ParseError) -> Self {
        Self::ParseURL(raw)
    }
}

impl Sender {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Sends the given Data as HTTP-JSON request to the Node with the
    /// given ID on the provided Path
    pub async fn send_data(
        &self,
        id: NodeId,
        path: &str,
        method: reqwest::Method,
        data: Vec<u8>,
    ) -> Result<reqwest::Response, SendError> {
        let addr = id_to_addr(id);
        let raw_url = format!("http://{}{}", addr, path);
        let url = reqwest::Url::parse(&raw_url)?;

        let request = self
            .client
            .request(method, url)
            .header("Content-Length", data.len())
            .header("Content-Type", "application/json")
            .body(data)
            .build()?;

        let response = self.client.execute(request).await?;
        if response.status() != reqwest::StatusCode::OK {
            return Err(SendError::ErrorResp);
        }

        Ok(response)
    }
}

#[async_trait]
impl async_raft::RaftNetwork<ClusterRequest> for Sender {
    async fn vote(&self, target: NodeId, rpc: VoteRequest) -> anyhow::Result<VoteResponse> {
        let data = serde_json::to_vec(&rpc)?;
        let response = self
            .send_data(target, "/vote", reqwest::Method::POST, data)
            .await?;

        let raw_body = response.bytes().await?;
        let body = serde_json::from_slice(&raw_body)?;
        Ok(body)
    }
    async fn append_entries(
        &self,
        target: NodeId,
        rpc: AppendEntriesRequest<ClusterRequest>,
    ) -> anyhow::Result<AppendEntriesResponse> {
        let data = serde_json::to_vec(&rpc)?;
        let response = self
            .send_data(target, "/entries/append", reqwest::Method::POST, data)
            .await?;

        let raw_body = response.bytes().await?;
        let body = serde_json::from_slice(&raw_body)?;
        Ok(body)
    }
    async fn install_snapshot(
        &self,
        target: NodeId,
        rpc: InstallSnapshotRequest,
    ) -> anyhow::Result<InstallSnapshotResponse> {
        let data = serde_json::to_vec(&rpc)?;
        let response = self
            .send_data(target, "/snapshot/install", reqwest::Method::POST, data)
            .await?;

        let raw_body = response.bytes().await?;
        let body = serde_json::from_slice(&raw_body)?;
        Ok(body)
    }
}

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
                serde_json::from_slice(request.body()).unwrap();
            let result = self.cluster.rpc_append_entries(rpc).await.unwrap();

            let body = serde_json::to_vec(&result).unwrap();
            return Some(body);
        }
        if SNAPSHOT.matches(request) {
            let rpc: InstallSnapshotRequest = serde_json::from_slice(request.body()).unwrap();
            let result = self.cluster.rpc_install_snapshot(rpc).await.unwrap();

            let body = serde_json::to_vec(&result).unwrap();
            return Some(body);
        }
        if VOTE.matches(request) {
            let rpc: VoteRequest = serde_json::from_slice(request.body()).unwrap();
            let result = self.cluster.rpc_vote(rpc).await.unwrap();

            let body = serde_json::to_vec(&result).unwrap();
            return Some(body);
        }
        None
    }

    async fn handle_leader(&self, request: &Request<'_>) -> Option<Vec<u8>> {
        if !self.cluster.is_leader().await {
            tracing::error!("Received Write Request as Leader, but node is not the Leader");
            return None;
        }

        if LEADER_WRITE.matches(request) {
            let req: ClusterRequest = serde_json::from_slice(request.body()).unwrap();
            return match self.cluster.write(req.domain, req.action).await {
                Ok(response) => {
                    let data = serde_json::to_vec(&response).unwrap();
                    Some(data)
                }
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
