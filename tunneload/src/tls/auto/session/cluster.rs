use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use async_raft::{
    raft::{
        AppendEntriesRequest, AppendEntriesResponse, ClientWriteRequest, ClientWriteResponse,
        InstallSnapshotRequest, InstallSnapshotResponse, VoteRequest, VoteResponse,
    },
    ClientWriteError, NodeId,
};

use serde::{Deserialize, Serialize};

mod network;
pub use network::addr_to_id;
use network::{Receiver, Sender};

mod statemachine;
mod storage;
use storage::Storage;

use crate::{
    configurator::{RuleList, ServiceList},
    tls::auto::NodeUpdateEvent,
};

use self::network::SendError;

use super::{AutoDiscover, CertificateQueue, ChallengeList};

#[derive(Debug)]
pub enum WriteError {
    MissingLeader,
    Raft(async_raft::RaftError),
    Forwarding(SendError),
    Serialize,
}

impl From<SendError> for WriteError {
    fn from(other: SendError) -> Self {
        Self::Forwarding(other)
    }
}

/// This represents a single Action of a Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterAction {
    /// This signals that the Certificate for a given Domain
    /// is missing and the Leader should start the process
    /// to generate the needed Certificate
    MissingCert,
    /// This signals some Data that should be used to Verify
    /// the ownership of a Domain for TLS-Certificates
    AddVerifyingData(Vec<(String, String)>),
    /// This signals that any Verfiying-Data that belongs to
    /// the Domain should be deleted and is no longer in use
    RemoveVerifyingData,
}

/// A general Request to perform some operation in the Cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRequest {
    domain: String,
    action: ClusterAction,
}

impl async_raft::AppData for ClusterRequest {}

/// The Response send back by the Cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResponse {}

impl async_raft::AppDataResponse for ClusterResponse {}

/// This struct should be used for all the interactions that relate
/// to a formed or currently forming Cluster
pub struct Cluster<D> {
    id: NodeId,
    raft: async_raft::Raft<ClusterRequest, ClusterResponse, Sender, Storage>,
    network_sender: Arc<Sender>,
    network_receiver: Arc<Receiver>,
    discover: Arc<D>,
}

impl<D> Debug for Cluster<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cluster (id = {})", self.id)
    }
}

impl<D> Cluster<D>
where
    D: AutoDiscover + Send + Sync + 'static,
{
    /// Creates the Cluster instance with all the other needed components
    pub async fn new(
        raw_discover: D,
        com_port: u16,
        challenges: ChallengeList,
        rules: RuleList,
        services: ServiceList,
        queue: CertificateQueue,
    ) -> Arc<Self> {
        let id = raw_discover.get_own_id().await;
        let config = async_raft::Config::build("tunneload-acme".to_owned())
            .heartbeat_interval(50)
            .election_timeout_min(400)
            .election_timeout_max(600)
            .validate()
            .expect("Creating Raft Configuration");

        let discover = Arc::new(raw_discover);

        let network_sender = Arc::new(Sender::new());
        let network_receiver = Arc::new(Receiver::new(com_port));

        let sm = statemachine::StateMachine::new(challenges, rules, services, queue);
        let storage = Storage::new(id, sm);

        let raft = async_raft::Raft::new(
            id,
            Arc::new(config),
            network_sender.clone(),
            Arc::new(storage),
        );

        Arc::new(Self {
            id,
            raft,
            network_sender,
            network_receiver,
            discover,
        })
    }

    /// Gets the Raft-Metrics
    pub fn metrics(&self) -> tokio::sync::watch::Receiver<async_raft::RaftMetrics> {
        self.raft.metrics()
    }

    /// Checks if the current Node is the Cluster-Leader
    pub async fn is_leader(&self) -> bool {
        self.raft.client_read().await.is_ok()
    }

    /// Adds a new node with the given ID to the Cluster
    #[tracing::instrument]
    pub async fn add_node(&self, id: NodeId) {
        tracing::info!("Adding Node: {}", id);

        if !self.is_leader().await {
            tracing::error!("Can't add Node to cluster, because this node is not the Leader");
            return;
        }

        if let Err(e) = self.raft.add_non_voter(id).await {
            tracing::error!("Could not Add new Node to Cluster: {:?}", e);
            return;
        }

        let all_nodes = self.discover.get_all_nodes().await;
        if let Err(e) = self.raft.change_membership(all_nodes).await {
            tracing::error!("Could not Update Memberships: {:?}", e);
            return;
        }

        tracing::info!("Added Node ({}) to the Cluster", id);
    }

    /// Attempts to remove the given Node from the Cluster
    #[tracing::instrument]
    pub async fn remove_node(&self, id: NodeId) {
        tracing::info!("Removing Node: {}", id);
    }

    #[tracing::instrument(skip(queue))]
    async fn listen_node_updates(
        self: Arc<Self>,
        mut queue: tokio::sync::mpsc::UnboundedReceiver<NodeUpdateEvent>,
    ) {
        loop {
            let event = match queue.recv().await {
                Some(e) => e,
                None => {
                    tracing::error!("Node Updates Queue has been closed");
                    return;
                }
            };

            match event {
                NodeUpdateEvent::Add(id) => {
                    self.add_node(id).await;
                }
                NodeUpdateEvent::Remove(id) => {
                    self.remove_node(id).await;
                }
            };
        }
    }

    /// Starts up the all the needed parts needed for the Cluster, but does
    /// not actually initalize and start the Cluster itself
    #[tracing::instrument]
    pub fn start(self: Arc<Self>) {
        tracing::info!("Starting Cluster");

        self.network_receiver.start(self.clone());

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::task::spawn(AutoDiscover::watch_nodes(self.discover.clone(), tx));
        tokio::task::spawn(self.listen_node_updates(rx));
    }

    /// Actually initalizes and starts the Cluster itself
    #[tracing::instrument]
    pub async fn initialize(&self) {
        tracing::info!("Initializing Cluster");

        let nodes = self.discover.get_all_nodes().await;
        tracing::info!("Initial Nodes: {:?}", nodes);

        if let Err(e) = self.raft.initialize(nodes).await {
            tracing::error!("Initializing Raft: {:?}", e);
        }
    }

    /// Attempts to execute a write on the Cluster and if the current
    /// Node is not the Leader, the Request will be forwarded to the
    /// current Leader for further processing
    pub async fn write(
        &self,
        domain: String,
        action: ClusterAction,
    ) -> Result<ClientWriteResponse<ClusterResponse>, WriteError> {
        let req = ClusterRequest { domain, action };

        let (req, target) = match self.raft.client_write(ClientWriteRequest::new(req)).await {
            Ok(resp) => return Ok(resp),
            Err(e) => match e {
                ClientWriteError::ForwardToLeader(req, target) => match target {
                    Some(leader_id) => (req, leader_id),
                    None => return Err(WriteError::MissingLeader),
                },
                ClientWriteError::RaftError(e) => return Err(WriteError::Raft(e)),
            },
        };

        let data = serde_json::to_vec(&req).map_err(|_| WriteError::Serialize)?;
        let response = self
            .network_sender
            .send_data(target, "/leader/write", reqwest::Method::POST, data)
            .await?;

        let body = response.bytes().await.map_err(|_| WriteError::Serialize)?;
        let result = serde_json::from_slice(&body).map_err(|_| WriteError::Serialize)?;
        Ok(result)
    }

    pub async fn rpc_append_entries(
        &self,
        rpc: AppendEntriesRequest<ClusterRequest>,
    ) -> Result<AppendEntriesResponse, async_raft::RaftError> {
        self.raft.append_entries(rpc).await
    }
    pub async fn rpc_install_snapshot(
        &self,
        rpc: InstallSnapshotRequest,
    ) -> Result<InstallSnapshotResponse, async_raft::RaftError> {
        self.raft.install_snapshot(rpc).await
    }
    pub async fn rpc_vote(&self, rpc: VoteRequest) -> Result<VoteResponse, async_raft::RaftError> {
        self.raft.vote(rpc).await
    }
}
