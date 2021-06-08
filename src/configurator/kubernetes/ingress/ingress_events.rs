use async_trait::async_trait;
use futures::FutureExt;
use k8s_openapi::api::extensions::v1beta1::Ingress;
use kube::{api::Meta, Api};

use crate::{
    configurator::parser::{self, EventEmitter, EventFuture, RawRuleConfig},
    util::kubernetes::watcher::{Event, Watcher},
};

/// The Event-Emitter for the Kubernetes-Ingress-Configuration
pub struct IngressEvents {
    client: kube::Client,
    namespace: String,
}

impl IngressEvents {
    /// Creates a new Instance of the Event-Emitter from the given
    /// initial Values
    pub fn new(client: kube::Client, namespace: String) -> Self {
        Self { client, namespace }
    }

    async fn rule_events(
        client: kube::Client,
        namespace: String,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawRuleConfig>>,
    ) {
        let api: Api<Ingress> = Api::namespaced(client, &namespace);

        let mut watcher = match Watcher::from_api(api, None).await {
            Ok(w) => w,
            Err(e) => {
                log::error!("Creating Watcher: {:?}", e);
                return;
            }
        };

        loop {
            let event = match watcher.next_event().await {
                Some(e) => e,
                None => {
                    log::error!("Watcher returned None");
                    return;
                }
            };

            match event {
                Event::Updated(rule) => {
                    if let Err(e) = sender.send(parser::Event::Update(RawRuleConfig {
                        config: serde_json::to_value(rule).unwrap(),
                    })) {
                        log::error!("Sending Event: {:?}", e);
                        return;
                    }
                }
                Event::Removed(rule) => {
                    let name = Meta::name(&rule);
                    if let Err(e) = sender.send(parser::Event::Remove(name)) {
                        log::error!("Sending Event: {:?}", e);
                        return;
                    }
                }
                Event::Restarted | Event::Other | Event::Started(_) => {}
            };
        }
    }
}

#[async_trait]
impl EventEmitter for IngressEvents {
    async fn rule_listener(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawRuleConfig>>,
    ) -> Option<EventFuture> {
        Some(Self::rule_events(self.client.clone(), self.namespace.clone(), sender).boxed())
    }
}
