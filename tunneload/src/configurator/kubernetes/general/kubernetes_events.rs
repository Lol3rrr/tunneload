use async_trait::async_trait;
use futures::FutureExt;
use general::Name;
use k8s_openapi::api::core::v1::{Endpoints, Secret};
use kube::Api;

use crate::{
    configurator::parser::{self, EventEmitter, EventFuture, RawServiceConfig, RawTLSConfig},
    util::kubernetes::{
        secret::tls_domain,
        watcher::{Event, Watcher},
    },
};

/// The Event-Emitter for the general Kubernetes-Configuration
pub struct KubernetesEvents {
    client: kube::Client,
    namespace: String,
}

impl KubernetesEvents {
    /// Creates a new Instace of the Event-Emitter from the given
    /// initial Values
    pub fn new(client: kube::Client, namespace: String) -> Self {
        Self { client, namespace }
    }

    async fn service_future(
        client: kube::Client,
        namespace: String,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawServiceConfig, Name>>,
    ) {
        let api: Api<Endpoints> = Api::namespaced(client, &namespace);
        let mut watcher = match Watcher::from_api(api, None).await {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("Could not create Watcher: {:?}", e);
                return;
            }
        };

        loop {
            let event = match watcher.next_event().await {
                Some(event) => event,
                None => {
                    tracing::error!("Could not get next Event");
                    return;
                }
            };

            match event {
                Event::Updated(updated) | Event::Removed(updated) => {
                    let value = match serde_json::to_value(updated) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    if let Err(e) =
                        sender.send(parser::Event::Update(RawServiceConfig { config: value }))
                    {
                        tracing::error!("Sending Event: {:?}", e);
                        return;
                    }
                }
                Event::Started(_) | Event::Restarted | Event::Other => {}
            };
        }
    }

    async fn tls_future(
        client: kube::Client,
        namespace: String,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawTLSConfig, String>>,
    ) {
        let api: Api<Secret> = Api::namespaced(client, &namespace);

        let mut watcher = match Watcher::from_api(api, None).await {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("Creating Watcher: {:?}", e);
                return;
            }
        };

        loop {
            let event = match watcher.next_event().await {
                Some(e) => e,
                None => {
                    tracing::error!("Watcher returned None");
                    return;
                }
            };

            match event {
                Event::Updated(secret) => {
                    if let Err(e) = sender.send(parser::Event::Update(RawTLSConfig {
                        config: serde_json::to_value(&secret)
                            .expect("Serializing should always work here"),
                    })) {
                        tracing::error!("Sending Event: {:?}", e);
                        return;
                    }
                }
                Event::Removed(secret) => {
                    let domain = match tls_domain(&secret) {
                        Some(d) => d,
                        None => continue,
                    };

                    if let Err(e) = sender.send(parser::Event::Remove(domain)) {
                        tracing::error!("Sending Event: {:?}", e);
                        return;
                    }
                }
                Event::Other | Event::Restarted | Event::Started(_) => {}
            };
        }
    }
}

#[async_trait]
impl EventEmitter for KubernetesEvents {
    async fn service_listener(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawServiceConfig, Name>>,
    ) -> Option<EventFuture> {
        Some(Self::service_future(self.client.clone(), self.namespace.clone(), sender).boxed())
    }

    async fn tls_listener(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawTLSConfig, String>>,
    ) -> Option<EventFuture> {
        Some(Self::tls_future(self.client.clone(), self.namespace.clone(), sender).boxed())
    }
}
