use serde::Deserialize;

/// A General definition of a single Kubernetes CRD
#[derive(Deserialize, Debug)]
pub struct Config<S> {
    /// The Api Version of this Ressource
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    /// The Ressource Kind of this Ressource
    pub kind: String,
    /// The Metadata of this Instance
    pub metadata: Metadata,
    /// The actual spec of the underlying Ressource
    pub spec: S,
}

/// The Metadata for a single Config
#[derive(Deserialize, Debug)]
pub struct Metadata {
    /// The Name of the Instance
    pub name: String,
    /// The Namespace of the Instance
    pub namespace: String,
}
