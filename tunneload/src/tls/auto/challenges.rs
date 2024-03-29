use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

/// This represents the individual states that a single
/// Challenge can be in
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeState {
    /// The missing Certificate was spotted but does not yet
    /// have any pending Challenge
    Pending,
    /// The Certificate is ready to be validated using the
    /// provided Data
    Data(Vec<(String, String)>),
    /// The Certificate was succesfully created
    Finished,
}

/// A Collection of all current ACME-Challenges
#[derive(Clone, Serialize, Deserialize)]
pub struct ChallengeList {
    entries: Arc<RwLock<HashMap<String, ChallengeState>>>,
}

impl ChallengeList {
    /// Creates a new empty Challenge-List
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Overwrites the previous state for the given Domain with the new
    /// given State
    pub fn update_state(&self, domain: String, n_state: ChallengeState) {
        let mut locked = match self.entries.write() {
            Ok(l) => l,
            Err(_) => {
                return;
            }
        };
        locked.insert(domain, n_state);
    }
    /// Removes the State Entry for the given Domain
    pub fn remove_state(&self, domain: &str) {
        let mut locked = match self.entries.write() {
            Ok(l) => l,
            Err(_) => {
                return;
            }
        };
        locked.remove(domain);
    }

    /// Gets the State of the Challenge for the given Domain
    pub fn get_state(&self, domain: &str) -> Option<ChallengeState> {
        let locked = match self.entries.read() {
            Ok(l) => l,
            Err(_) => {
                return None;
            }
        };
        locked.get(domain).cloned()
    }

    /// Clones the entire underlying Hashmap
    pub fn clone_map(&self) -> HashMap<String, ChallengeState> {
        let locked = match self.entries.read() {
            Ok(l) => l,
            Err(_) => {
                return HashMap::new();
            }
        };
        locked.clone()
    }
    /// Replaces the underlying Hashmap with the given Map
    pub fn set_map(&self, n_map: HashMap<String, ChallengeState>) {
        let mut locked = match self.entries.write() {
            Ok(l) => l,
            Err(_) => {
                return;
            }
        };
        *locked = n_map;
    }
}

impl Default for ChallengeList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ChallengeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entries = self.entries.read().expect("Locking for Debug output");
        let map = &*entries;
        write!(f, "ChallengeList: {:?}", map)
    }
}
