use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config<S> {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: S,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub namespace: String,
}
