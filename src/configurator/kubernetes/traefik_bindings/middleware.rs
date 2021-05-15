use crate::configurator::kubernetes::general_crd;

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
pub struct MiddlewareSpec {}

/// The underlying Config for Middlewares
pub type Config = general_crd::Config<std::collections::BTreeMap<String, serde_json::Value>>;
