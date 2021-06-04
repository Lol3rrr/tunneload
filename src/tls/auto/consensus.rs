use serde::{Deserialize, Serialize};

mod network;
pub use network::{addr_to_id, Network, NetworkReceiver};

mod storage;
pub use storage::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    MissingCert,
    VerifyingData(Vec<(String, String)>),
    Failed,
    Finish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub domain_name: String,
    pub action: Action,
}

impl async_raft::AppData for Request {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {}

impl async_raft::AppDataResponse for Response {}
