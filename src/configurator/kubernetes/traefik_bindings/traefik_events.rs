use async_trait::async_trait;
use futures::FutureExt;
use kube::{api::Meta, Api};

use crate::{
    configurator::{
        kubernetes::traefik_bindings::{self, ingressroute::IngressRoute, middleware::Middleware},
        parser::{self, EventEmitter, EventFuture, RawMiddlewareConfig, RawRuleConfig},
    },
    util::kubernetes::watcher::{Event, Watcher},
};

/// This is the Event-Emitter for the Kubernetes-Traefik-Configuration
pub struct TraefikEvents {
    client: kube::Client,
    namespace: String,
}

impl TraefikEvents {
    /// Creates a new Instance of the Event-Emitter using the given
    /// initial Values
    pub fn new(client: kube::Client, namespace: String) -> Self {
        Self { client, namespace }
    }

    async fn middleware_events(
        client: kube::Client,
        namespace: String,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawMiddlewareConfig>>,
    ) {
        let api: Api<Middleware> = Api::namespaced(client, &namespace);
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
                    tracing::error!("Watcher returned None-Event");
                    return;
                }
            };

            match event {
                Event::Updated(mid) => {
                    let metadata = mid.metadata;
                    if let Some(raw_annotations) = metadata.annotations {
                        let last_applied = raw_annotations
                            .get("kubectl.kubernetes.io/last-applied-configuration")
                            .unwrap();

                        let current_config: traefik_bindings::middleware::Config =
                            serde_json::from_str(last_applied).unwrap();

                        let name = current_config.metadata.name;
                        for (key, value) in current_config.spec.iter() {
                            if let Err(e) =
                                sender.send(parser::Event::Update(RawMiddlewareConfig {
                                    name: name.clone(),
                                    action_name: key.clone(),
                                    config: value.clone(),
                                }))
                            {
                                tracing::error!("Sending Event: {:?}", e);
                                return;
                            }
                        }
                    }
                }
                Event::Removed(mid) => {
                    let metadata = mid.metadata;
                    if let Some(raw_annotations) = metadata.annotations {
                        let last_applied = raw_annotations
                            .get("kubectl.kubernetes.io/last-applied-configuration")
                            .unwrap();

                        let current_config: traefik_bindings::middleware::Config =
                            serde_json::from_str(last_applied).unwrap();

                        let name = current_config.metadata.name;

                        if let Err(e) = sender.send(parser::Event::Remove(name)) {
                            tracing::error!("Sending Event: {:?}", e);
                            return;
                        }
                    }
                }
                Event::Started(_) | Event::Restarted | Event::Other => {}
            };
        }
    }

    async fn rule_events(
        client: kube::Client,
        namespace: String,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawRuleConfig>>,
    ) {
        let api: Api<IngressRoute> = Api::namespaced(client, &namespace);

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
                Event::Updated(rule) => {
                    let metadata = rule.metadata;
                    if let Some(raw_annotations) = metadata.annotations {
                        let last_applied = raw_annotations
                            .get("kubectl.kubernetes.io/last-applied-configuration")
                            .unwrap();

                        let current_config: serde_json::Value =
                            serde_json::from_str(last_applied).unwrap();

                        if let Err(e) = sender.send(parser::Event::Update(RawRuleConfig {
                            config: current_config,
                        })) {
                            tracing::error!("Sending Event: {:?}", e);
                            return;
                        }
                    }
                }
                Event::Removed(rule) => {
                    let name = Meta::name(&rule);
                    if let Err(e) = sender.send(parser::Event::Remove(name)) {
                        tracing::error!("Sending Event: {:?}", e);
                        return;
                    }
                }
                Event::Restarted | Event::Other | Event::Started(_) => {}
            };
        }
    }
}

#[async_trait]
impl EventEmitter for TraefikEvents {
    async fn middleware_listener(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawMiddlewareConfig>>,
    ) -> Option<EventFuture> {
        Some(Self::middleware_events(self.client.clone(), self.namespace.clone(), sender).boxed())
    }

    async fn rule_listener(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawRuleConfig>>,
    ) -> Option<EventFuture> {
        Some(Self::rule_events(self.client.clone(), self.namespace.clone(), sender).boxed())
    }
}
