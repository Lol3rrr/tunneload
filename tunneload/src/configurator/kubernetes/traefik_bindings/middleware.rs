// These are only allowed here because the Macros otherwise cause warnings that can not be fixed
#![allow(clippy::disallowed_methods)]
#![allow(missing_docs)]

use std::collections::BTreeMap;

use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The Spec for Traefik based Middleware ressources
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "traefik.containo.us",
    version = "v1alpha1",
    kind = "Middleware",
    plural = "middlewares",
    namespaced
)]
pub struct MiddlewareSpec {
    /// The Strip-Prefix config options
    #[serde(rename = "stripPrefix", skip_serializing_if = "Option::is_none")]
    pub strip_prefix: Option<StripPrefix>,
    /// The Headers/CORS config options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, Vec<String>>>,
    /// The Compress config options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compress: Option<Compress>,
    /// The Basic-Auth config options
    #[serde(rename = "basicAuth", skip_serializing_if = "Option::is_none")]
    pub basic_auth: Option<BasicAuth>,
}

/// The Strip-Prefix Configuration
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct StripPrefix {
    /// All the Prefixes to remove from the Requests
    pub prefixes: Vec<String>,
}

/// The Compress Configuration
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct Compress {}

/// The Basic-Auth Configuration
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct BasicAuth {
    secret: String,
}
