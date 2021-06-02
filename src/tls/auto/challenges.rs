use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeState {
    Pending,
    Data(Vec<(String, String)>),
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeList {
    entries: Arc<RwLock<HashMap<String, ChallengeState>>>,
}

impl ChallengeList {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn update_state(&self, domain: String, n_state: ChallengeState) {
        self.entries.write().unwrap().insert(domain, n_state);
    }
    pub fn remove_state(&self, domain: &str) {
        self.entries.write().unwrap().remove(domain);
    }

    pub fn get_state(&self, domain: &str) -> Option<ChallengeState> {
        self.entries
            .write()
            .unwrap()
            .get(domain)
            .map(|v| v.to_owned())
    }

    pub fn clone_map(&self) -> HashMap<String, ChallengeState> {
        self.entries.read().unwrap().clone()
    }
    pub fn set_map(&self, n_map: HashMap<String, ChallengeState>) {
        *self.entries.write().unwrap() = n_map;
    }

    pub fn get_challenge(&self, domain: &str) -> Option<ChallengeState> {
        self.entries.read().unwrap().get(domain).map(|v| v.clone())
    }
}
