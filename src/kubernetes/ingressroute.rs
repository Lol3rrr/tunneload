use crate::kubernetes::general_crd;

use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "traefik.containo.us",
    version = "v1alpha1",
    kind = "IngressRoute",
    plural = "ingressroutes",
    namespaced
)]
pub struct IngressRouteSpec {}

pub type Config = general_crd::Config<Spec>;

#[derive(Deserialize, Debug)]
pub struct Spec {
    #[serde(rename = "entryPoints")]
    pub entry_points: Vec<String>,
    pub routes: Vec<Route>,
    pub tls: Option<TLS>,
}

#[derive(Deserialize, Debug)]
pub struct TLS {
    #[serde(rename = "secretName")]
    pub secret_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Route {
    pub kind: String,
    #[serde(rename = "match")]
    pub rule: String,
    #[serde(default)]
    pub middlewares: Vec<Middleware>,
    pub priority: Option<u32>,
    pub services: Vec<Service>,
}

#[derive(Deserialize, Debug)]
pub struct Middleware {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Service {
    pub name: String,
    pub port: Option<u32>,
}
