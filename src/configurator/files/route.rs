use serde::{Deserialize, Serialize};

/// The Service Configuration for a given Route
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigService {
    /// The Name of the Service
    pub name: String,
    /// An optional List of addresses that should be used for
    /// this service
    pub addresses: Option<Vec<String>>,
}

/// The Rule Configuration for a single Rule
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigRoute {
    /// The Name of the Rule itself
    pub name: String,
    /// The Priority of the Rule
    #[serde(default = "default_priority")]
    pub priority: u32,
    /// The actual Matcher-Rule to use
    pub rule: String,
    /// The target Service for all matching Requests
    pub service: String,
    /// An opitonal List of all Middlewares for this Rule
    pub middleware: Option<Vec<String>>,
}

fn default_priority() -> u32 {
    1
}
