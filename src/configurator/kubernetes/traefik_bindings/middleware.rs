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
    #[serde(rename = "stripPrefix", skip_serializing_if = "Option::is_none")]
    pub strip_prefix: Option<StripPrefix>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compress: Option<Compress>,
    #[serde(rename = "basicAuth", skip_serializing_if = "Option::is_none")]
    pub basic_auth: Option<BasicAuth>,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct StripPrefix {
    pub prefixes: Vec<String>,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct Compress {}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct BasicAuth {
    secret: String,
}
