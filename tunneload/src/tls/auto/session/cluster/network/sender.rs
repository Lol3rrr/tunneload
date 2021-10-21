use async_raft::{
    raft::{
        AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest,
        InstallSnapshotResponse, VoteRequest, VoteResponse,
    },
    NodeId,
};
use async_trait::async_trait;

use crate::tls::auto::session::cluster::{network::id_to_addr, ClusterRequest};

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
