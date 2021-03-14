use crate::configurator::Configurator;
use crate::configurator::{
    kubernetes::{ingress, traefik_bindings},
    MiddlewareList, RuleList,
};
use crate::rules::{Middleware, Rule};
use crate::{
    configurator::kubernetes::general::load_tls, configurator::ServiceList, rules::Service,
};

use crate::configurator::kubernetes::general::{parse_endpoint, Event, Watcher};
use async_trait::async_trait;
use futures::{Future, FutureExt};
use kube::{api::ListParams, Api, Client};
use tokio::join;

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

        let mut stream = match Watcher::from_api(endpoints, Some(lp)).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("Creating Stream: {}", e);
                return;
            }
        };

        loop {
            let event = match stream.next_event().await {
                Some(e) => e,
                None => {
                    return;
                }
            };

            match event {
                // Handle Update and Remove the same because otherwise removing
                // a single instance from the Endpoint would delete all the
                // endpoints in the list and make the entire service unavailable
                Event::Updated(srv) | Event::Removed(srv) => {
                    // Parse the received Event
                    let service = match parse_endpoint(&srv) {
                        Some(s) => s,
                        None => {
                            log::error!("Could not Parse-Service: {:?}", srv);
                            continue;
                        }
                    };

                    log::info!("Updated Service: {:?}", service);

                    // Update the service to reflect the newest state
                    services.set_service(service);
                }
                Event::Other => {}
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

    async fn load_tls(&mut self) -> Vec<(String, rustls::sign::CertifiedKey)> {
        load_tls(self.client.clone(), &self.namespace).await
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
        async fn placeholder() {}

        let traefik_based = if self.use_traefik {
            traefik_bindings::events::listen_middlewares(
                self.client.clone(),
                self.namespace.clone(),
                middlewares,
            )
            .boxed()
        } else {
            placeholder().boxed()
        };

        async fn run(futures: std::pin::Pin<Box<dyn Future<Output = ()> + 'static + Send>>) {
            join!(futures);
        }

        Box::pin(run(traefik_based))
    }

    fn get_rules_event_listener(
        &mut self,
        middlewares: MiddlewareList,
        services: ServiceList,
        rules: RuleList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        async fn placeholder() {}

        let traefik_based = if self.use_traefik {
            traefik_bindings::events::listen_rules(
                self.client.clone(),
                self.namespace.clone(),
                middlewares,
                services,
                rules,
            )
            .boxed()
        } else {
            placeholder().boxed()
        };

        async fn run(futures: std::pin::Pin<Box<dyn Future<Output = ()> + 'static + Send>>) {
            join!(futures);
        }

        Box::pin(run(traefik_based))
    }
}
