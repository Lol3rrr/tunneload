use crate::configurator::Configurator;
use crate::configurator::{
    kubernetes::{ingress, traefik_bindings},
    MiddlewareList,
};
use crate::rules::{Middleware, Rule};
use crate::{
    configurator::kubernetes::general::load_tls, configurator::ServiceList, rules::Service,
};

use crate::configurator::kubernetes::general::parse_endpoint;
use async_trait::async_trait;
use futures::{Future, StreamExt, TryStreamExt};
use kube::{
    api::{ListParams, Meta, WatchEvent},
    Api, Client,
};
use traefik_bindings::parse::parse_middleware;

use crate::configurator::ConfigItem;

use super::general::load_services;

#[derive(Clone)]
pub struct Loader {
    client: Client,
    namespace: String,
    use_traefik: bool,
    use_ingress: bool,
    ingress_priority: u32,
}

impl Loader {
    pub async fn new(namespace: String) -> Self {
        let client = Client::try_default().await.unwrap();

        Self {
            client,
            namespace,
            use_traefik: false,
            use_ingress: false,
            ingress_priority: 100,
        }
    }
    pub fn enable_traefik(&mut self) {
        self.use_traefik = true;
    }
    pub fn enable_ingress(&mut self) {
        self.use_ingress = true;
    }
    pub fn set_ingress_priority(&mut self, n_priority: u32) {
        self.ingress_priority = n_priority;
    }

    async fn service_events(client: kube::Client, namespace: String, services: ServiceList) {
        //let events: Api<Event> = Api::namespaced(self.client.clone(), &self.namespace);
        let endpoints: Api<k8s_openapi::api::core::v1::Endpoints> =
            Api::namespaced(client, &namespace);

        let lp = ListParams::default();

        let mut stream = match endpoints.watch(&lp, "0").await {
            Ok(s) => s.boxed(),
            Err(e) => {
                log::error!("Creating Stream: {}", e);
                return;
            }
        };

        loop {
            let raw_srv = match stream.try_next().await {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Getting Kubernetes-Service-Event: {}", e);
                    continue;
                }
            };

            let srv = match raw_srv {
                Some(s) => s,
                None => {
                    continue;
                }
            };

            match srv {
                WatchEvent::Added(srv) | WatchEvent::Modified(srv) => {
                    // Parse the received Event
                    let service = match parse_endpoint(&srv) {
                        Some(s) => s,
                        None => continue,
                    };

                    log::info!("Updating Service: {}", service.name());

                    // Update the service to reflect the newest state
                    services.set_service(service);
                }
                WatchEvent::Deleted(srv) => {
                    let name = Meta::name(&srv);

                    log::info!("Clearing Service: {}", name);

                    // Replace the service with an empty one
                    services.set_service(Service::new(name, vec![]));
                }
                _ => {
                    log::error!("Unknown OP: {:?}", srv);
                }
            };
        }
    }

    async fn traefik_middleware_events(
        client: kube::Client,
        namespace: String,
        middlewares: MiddlewareList,
    ) {
        let middleware_crds: Api<traefik_bindings::middleware::Middleware> =
            Api::namespaced(client.clone(), &namespace);

        let lp = ListParams::default();

        let mut stream = match middleware_crds.watch(&lp, "0").await {
            Ok(s) => s.boxed(),
            Err(e) => {
                log::error!("Creating Stream: {}", e);
                return;
            }
        };

        loop {
            let raw_middleware = match stream.try_next().await {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Getting Kubernetes-Middleware-Event: {}", e);
                    continue;
                }
            };

            let middleware = match raw_middleware {
                Some(m) => m,
                None => {
                    continue;
                }
            };

            match middleware {
                WatchEvent::Added(mid) | WatchEvent::Modified(mid) => {
                    let metadata = mid.metadata;
                    if let Some(raw_annotations) = metadata.annotations {
                        let last_applied = raw_annotations
                            .get("kubectl.kubernetes.io/last-applied-configuration")
                            .unwrap();

                        let current_config: traefik_bindings::middleware::Config =
                            serde_json::from_str(last_applied).unwrap();

                        let mut res_middlewares = parse_middleware(
                            Some(client.clone()),
                            Some(&namespace),
                            current_config,
                        )
                        .await;

                        for tmp in res_middlewares.drain(..) {
                            log::info!("Updated Middleware: {}", tmp.name());

                            middlewares.set_middleware(tmp);
                        }
                    }
                }
                WatchEvent::Deleted(srv) => {
                    let name = Meta::name(&srv);

                    log::info!("Deleting Middleware: {}", name);

                    middlewares.remove_middleware(&name);
                }
                _ => {
                    log::error!("Unknown OP: {:?}", middleware);
                }
            };
        }
    }
}

#[async_trait]
impl Configurator for Loader {
    async fn load_services(&mut self) -> Vec<Service> {
        load_services(self.client.clone(), &self.namespace).await
    }

    async fn load_middleware(&mut self) -> Vec<Middleware> {
        let mut result = Vec::new();

        if self.use_traefik {
            let mut traefik =
                traefik_bindings::load_middlewares(self.client.clone(), &self.namespace).await;
            result.append(&mut traefik);
        }

        result
    }

    async fn load_rules(
        &mut self,
        middlewares: &MiddlewareList,
        services: &ServiceList,
    ) -> Vec<Rule> {
        let mut result = Vec::new();

        if self.use_traefik {
            let mut traefik = traefik_bindings::load_routes(
                self.client.clone(),
                &self.namespace,
                middlewares,
                services,
            )
            .await;
            result.append(&mut traefik);
        }

        if self.use_ingress {
            let mut ingress_routes =
                ingress::load_routes(self.client.clone(), &self.namespace, self.ingress_priority)
                    .await;
            result.append(&mut ingress_routes);
        }

        result
    }

    async fn load_tls(&mut self, rules: &[Rule]) -> Vec<(String, rustls::sign::CertifiedKey)> {
        load_tls(self.client.clone(), &self.namespace, rules).await
    }

    fn get_serivce_event_listener(
        &mut self,
        services: ServiceList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        Box::pin(Self::service_events(
            self.client.clone(),
            self.namespace.clone(),
            services,
        ))
    }

    fn get_middleware_event_listener(
        &mut self,
        middlewares: MiddlewareList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        Box::pin(Self::traefik_middleware_events(
            self.client.clone(),
            self.namespace.clone(),
            middlewares,
        ))
    }
}
