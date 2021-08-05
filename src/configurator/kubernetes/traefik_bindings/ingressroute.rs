use crate::configurator::kubernetes::general_crd;

use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The actual Spec
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "traefik.containo.us",
    version = "v1alpha1",
    kind = "IngressRoute",
    plural = "ingressroutes",
    namespaced
)]
pub struct Spec {
    /// All the Entrypoints that should lead to this Route
    #[serde(rename = "entryPoints")]
    pub entry_points: Option<Vec<String>>,
    /// All the Routes assosicated with the given Rule
    pub routes: Vec<Route>,
    /// The TLS-Config for the Routes
    pub tls: Option<Tls>,
}

/// The Traefik TLS configuration
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct Tls {
    /// The Name of the Kubernetes Secret for the TLS-Certs
    #[serde(rename = "secretName")]
    pub secret_name: Option<String>,
}

/// The actual Traefik Route
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct Route {
    /// The Kind of Route
    pub kind: String,
    #[serde(rename = "match")]
    /// The Rules used to determine if a Request matches
    /// this Route
    pub rule: String,
    #[serde(default)]
    /// The Middlewares that should be applied
    pub middlewares: Vec<Middleware>,
    /// The Priority of the Route
    pub priority: Option<u32>,
    /// The Target service of this Route
    pub services: Vec<Service>,
}

/// The Traefik Middleware configuration
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct Middleware {
    /// The registered Name of the Middleware
    pub name: String,
}

/// The Traefik target service configuration
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct Service {
    /// The name of the Service
    pub name: String,
    /// The Port to which the requests should be
    /// forwarded to
    pub port: Option<u32>,
}
