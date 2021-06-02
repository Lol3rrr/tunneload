use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_raft::{
    raft::{
        AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest,
        InstallSnapshotResponse, VoteRequest, VoteResponse,
    },
    NodeId,
};
use async_trait::async_trait;

use super::Request;

/// The RaftNetwork implementation
pub struct Network {
    nodes: Arc<Mutex<HashMap<NodeId, String>>>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_node(&self, id: NodeId, address: String) {
        self.nodes.lock().unwrap().insert(id, address);
    }
}

#[async_trait]
impl async_raft::RaftNetwork<Request> for Network {
    async fn append_entries(
        &self,
        target: NodeId,
        rpc: AppendEntriesRequest<Request>,
    ) -> anyhow::Result<AppendEntriesResponse> {
        let node_url = match self.nodes.lock().unwrap().get(&target) {
            Some(node) => node.clone(),
            None => todo!("Handle Error correctly"),
        };
        let final_url = reqwest::Url::parse(&format!("{}/entries/append", node_url))?;

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
        let node_url = match self.nodes.lock().unwrap().get(&target) {
            Some(node) => node.clone(),
            None => todo!("Handle Error correctly"),
        };
        let final_url = reqwest::Url::parse(&format!("{}/snapshot/install", node_url))?;

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
        let node_url = match self.nodes.lock().unwrap().get(&target) {
            Some(node) => node.clone(),
            None => todo!("Handle Error correctly"),
        };
        let final_url = reqwest::Url::parse(&format!("{}/vote", node_url))?;

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
