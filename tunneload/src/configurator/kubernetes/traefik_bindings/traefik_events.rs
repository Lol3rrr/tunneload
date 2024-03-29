use async_trait::async_trait;
use futures::FutureExt;
use general::{Group, Name};
use kube::{api::ResourceExt, Api};

use crate::{
    configurator::{
        kubernetes::traefik_bindings::{ingressroute::IngressRoute, middleware::Middleware},
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
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawMiddlewareConfig, Name>>,
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
                    let metadata = &mid.metadata;
                    let name = match metadata.name.as_ref() {
                        Some(n) => n.to_owned(),
                        None => continue,
                    };
                    let namespace = metadata
                        .namespace
                        .clone()
                        .unwrap_or_else(|| "default".to_owned());

                    let current_config = match serde_json::to_value(mid) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    let spec = match current_config.as_object() {
                        Some(s) => s,
                        None => continue,
                    };

                    for (key, value) in spec.iter() {
                        let ev_name = Name::new(
                            &name,
                            Group::Kubernetes {
                                namespace: namespace.clone(),
                            },
                        );
                        if let Err(e) = sender.send(parser::Event::Update(RawMiddlewareConfig {
                            name: ev_name,
                            action_name: key.clone(),
                            config: value.clone(),
                        })) {
                            tracing::error!("Sending Event: {:?}", e);
                            return;
                        }
                    }
                }
                Event::Removed(mid) => {
                    let metadata = mid.metadata;
                    let name = match metadata.name {
                        Some(n) => n,
                        None => continue,
                    };
                    let namespace = metadata.namespace.unwrap_or_else(|| "default".to_string());

                    let ev_name = Name::new(name, Group::Kubernetes { namespace });
                    if let Err(e) = sender.send(parser::Event::Remove(ev_name)) {
                        tracing::error!("Sending Event: {:?}", e);
                        return;
                    }
                }
                Event::Started(_) | Event::Restarted | Event::Other => {}
            };
        }
    }

    async fn rule_events(
        client: kube::Client,
        namespace: String,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawRuleConfig, Name>>,
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
                    let current_config = match serde_json::to_value(rule) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    if let Err(e) = sender.send(parser::Event::Update(RawRuleConfig {
                        config: current_config,
                    })) {
                        tracing::error!("Sending Event: {:?}", e);
                        return;
                    }
                }
                Event::Removed(rule) => {
                    let name = ResourceExt::name(&rule);
                    let namespace =
                        ResourceExt::namespace(&rule).unwrap_or_else(|| "default".to_string());

                    let ev_name = Name::new(name, Group::Kubernetes { namespace });
                    if let Err(e) = sender.send(parser::Event::Remove(ev_name)) {
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
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawMiddlewareConfig, Name>>,
    ) -> Option<EventFuture> {
        Some(Self::middleware_events(self.client.clone(), self.namespace.clone(), sender).boxed())
    }

    async fn rule_listener(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawRuleConfig, Name>>,
    ) -> Option<EventFuture> {
        Some(Self::rule_events(self.client.clone(), self.namespace.clone(), sender).boxed())
    }
}
