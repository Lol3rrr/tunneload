use std::sync::Arc;

use async_trait::async_trait;
use futures::FutureExt;
use k8s_openapi::api::core::v1::Endpoints;
use kube::Api;

use crate::configurator::parser::{self, EventEmitter, EventFuture, RawServiceConfig};

use super::{Event, Watcher};

pub struct KubernetesEvents {
    client: kube::Client,
    namespace: String,
}

impl KubernetesEvents {
    pub fn new(client: kube::Client, namespace: String) -> Self {
        Self { client, namespace }
    }

    async fn service_future(
        client: kube::Client,
        namespace: String,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawServiceConfig>>,
    ) {
        let api: Api<Endpoints> = Api::namespaced(client, &namespace);
        let mut watcher = match Watcher::from_api(api, None).await {
            Ok(w) => w,
            Err(e) => {
                log::error!("Could not create Watcher: {:?}", e);
                return;
            }
        };

        loop {
            let event = match watcher.next_event().await {
                Some(event) => event,
                None => {
                    log::error!("Could not get next Event");
                    return;
                }
            };

            match event {
                Event::Updated(updated) | Event::Removed(updated) => {
                    let value = serde_json::to_value(updated).unwrap();
                    if let Err(e) =
                        sender.send(parser::Event::Update(RawServiceConfig { config: value }))
                    {
                        log::error!("Sending Event: {:?}", e);
                        return;
                    }
                }
                Event::Started(_) | Event::Restarted | Event::Other => {}
            };
        }
    }
}

#[async_trait]
impl EventEmitter for KubernetesEvents {
    async fn service_listener(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawServiceConfig>>,
    ) -> EventFuture {
        Self::service_future(self.client.clone(), self.namespace.clone(), sender).boxed()
    }
}
