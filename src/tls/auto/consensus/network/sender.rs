use async_raft::{
    raft::{
        AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest,
        InstallSnapshotResponse, VoteRequest, VoteResponse,
    },
    NodeId,
};
use async_trait::async_trait;

use crate::tls::auto::consensus::network::id_to_addr;

use super::Request;

/// The RaftNetwork implementation
pub struct Network {
    port: u16,
}

impl Network {
    pub fn new() -> Self {
        Self { port: 8080 }
    }
}

#[async_trait]
impl async_raft::RaftNetwork<Request> for Network {
    async fn append_entries(
        &self,
        target: NodeId,
        rpc: AppendEntriesRequest<Request>,
    ) -> anyhow::Result<AppendEntriesResponse> {
        let addr = id_to_addr(target, self.port);
        let final_url = reqwest::Url::parse(&format!("{}/entries/append", addr))?;

        let req_client = reqwest::Client::new();
        let mut req_builder = req_client.request(reqwest::Method::POST, final_url);
        req_builder = req_builder.json(&rpc);

        let request = req_builder.build()?;

        let response = req_client.execute(request).await?;
        let raw_body = response.bytes().await?;

        let body = serde_json::from_slice(&raw_body)?;
        Ok(body)
    }

    async fn install_snapshot(
        &self,
        target: NodeId,
        rpc: InstallSnapshotRequest,
    ) -> anyhow::Result<InstallSnapshotResponse> {
        let addr = id_to_addr(target, self.port);
        let final_url = reqwest::Url::parse(&format!("{}/snapshot/install", addr))?;

        let req_client = reqwest::Client::new();
        let mut req_builder = req_client.request(reqwest::Method::POST, final_url);
        req_builder = req_builder.json(&rpc);

        let request = req_builder.build()?;

        let response = req_client.execute(request).await?;
        let raw_body = response.bytes().await?;

        let body = serde_json::from_slice(&raw_body)?;
        Ok(body)
    }

    async fn vote(&self, target: NodeId, rpc: VoteRequest) -> anyhow::Result<VoteResponse> {
        let addr = id_to_addr(target, self.port);
        let final_url = reqwest::Url::parse(&format!("{}/vote", addr))?;

        let req_client = reqwest::Client::new();
        let mut req_builder = req_client.request(reqwest::Method::POST, final_url);
        req_builder = req_builder.json(&rpc);

        let request = req_builder.build()?;

        let response = req_client.execute(request).await?;
        let raw_body = response.bytes().await?;

        let body = serde_json::from_slice(&raw_body)?;
        Ok(body)
    }
}
