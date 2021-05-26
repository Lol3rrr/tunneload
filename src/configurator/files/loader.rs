use crate::configurator::ActionPluginList;
use crate::rules::{Middleware, Rule};
use crate::tls;
use crate::{configurator::files, configurator::ServiceList, rules::Service};
use crate::{
    configurator::{Configurator, MiddlewareList, RuleList},
    internal_services::DashboardEntity,
};

use async_trait::async_trait;
use futures::Future;
use serde_json::json;

use std::{fs, time::Duration};

use super::FileParser;

mod events;

/// The actual Datatype that is used to load the Data from the
/// a specific File/Folder
pub struct Loader {
    /// The Path for the Config-Files
    path: String,
    /// The Parser
    parser: FileParser,
}

impl Loader {
    /// Creates a new Loader instance with the given Path
    /// as the Config-Path
    pub fn new(path: String) -> Self {
        Self {
            path,
            parser: FileParser::new(),
        }
    }
}

#[async_trait]
impl Configurator for Loader {
    // TODO
    async fn load_services(&mut self) -> Vec<Service> {
        Vec::new()
    }

    async fn load_middleware(&mut self, action_plugins: &ActionPluginList) -> Vec<Middleware> {
        let metadata = fs::metadata(&self.path).unwrap();
        if metadata.is_file() {
            files::load_middlewares(&self.path, &self.parser, action_plugins).await
        } else {
            let mut tmp = Vec::new();
            for entry in fs::read_dir(&self.path).unwrap() {
                let mut result =
                    files::load_middlewares(entry.unwrap().path(), &self.parser, action_plugins)
                        .await;
                tmp.append(&mut result);
            }
            tmp
        }
    }

    async fn load_rules(
        &mut self,
        middlewares: &MiddlewareList,
        services: &ServiceList,
    ) -> Vec<Rule> {
        let metadata = fs::metadata(&self.path).unwrap();
        if metadata.is_file() {
            files::load_routes(&self.path, middlewares, services)
        } else {
            let mut tmp = Vec::new();
            for entry in fs::read_dir(&self.path).unwrap() {
                let mut result = files::load_routes(entry.unwrap().path(), middlewares, services);
                tmp.append(&mut result);
            }
            tmp
        }
    }

    async fn load_tls(&mut self) -> Vec<(String, rustls::sign::CertifiedKey)> {
        Vec::new()
    }

    fn get_serivce_event_listener(
        &mut self,
        services: ServiceList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        // TODO
        // Actually listen to file-events
        async fn run() {
            loop {
                tokio::time::sleep(Duration::new(60, 0)).await;
            }
        }

        // This only happens because we dont want the List
        // to be dropped
        std::mem::forget(services);

        Box::pin(run())
    }

    fn get_middleware_event_listener(
        &mut self,
        middlewares: MiddlewareList,
        action_plugins: ActionPluginList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        async fn listen_events(
            path: String,
            middlewares: MiddlewareList,
            parser: FileParser,
            action_plugins: ActionPluginList,
        ) {
            let watcher = match events::CustomWatcher::new(path) {
                Some(w) => w,
                None => {
                    log::error!("Failed to create Middleware-File-Watcher");
                    return;
                }
            };

            for path in watcher {
                for tmp in files::load_middlewares(path, &parser, &action_plugins)
                    .await
                    .drain(..)
                {
                    middlewares.set_middleware(tmp);
                }
            }
        }

        async fn run(
            path: String,
            middlewares: MiddlewareList,
            parser: FileParser,
            action_plugins: ActionPluginList,
        ) {
            let handle = tokio::task::spawn_blocking(move || {
                futures::executor::block_on(listen_events(
                    path,
                    middlewares,
                    parser,
                    action_plugins,
                ));
            });

            match handle.await {
                Ok(_) => {
                    log::info!("Middleware-File-Watcher stopped");
                }
                Err(e) => {
                    log::info!("Middleware-File-Watcher stopped: {:?}", e);
                }
            };
        }

        Box::pin(run(
            self.path.clone(),
            middlewares,
            self.parser.clone(),
            action_plugins,
        ))
    }

    fn get_rules_event_listener(
        &mut self,
        middlewares: MiddlewareList,
        services: ServiceList,
        rules: RuleList,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        fn listen_events(
            path: String,
            rules: RuleList,
            middlewares: MiddlewareList,
            services: ServiceList,
        ) {
            let watcher = match events::CustomWatcher::new(path) {
                Some(w) => w,
                None => {
                    log::error!("Failed to create Middleware-File-Watcher");
                    return;
                }
            };

            for path in watcher {
                for tmp in files::load_routes(path, &middlewares, &services).drain(..) {
                    rules.set_rule(tmp);
                }
            }
        }

        async fn run(
            path: String,
            rules: RuleList,
            middlewares: MiddlewareList,
            services: ServiceList,
        ) {
            let handle = tokio::task::spawn_blocking(move || {
                listen_events(path, rules, middlewares, services);
            });

            match handle.await {
                Ok(_) => {
                    log::info!("Rules-File-Watcher stopped");
                }
                Err(e) => {
                    log::info!("Rules-File-Watcher stopped: {:?}", e);
                }
            };
        }

        Box::pin(run(self.path.clone(), rules, middlewares, services))
    }

    fn get_tls_event_listener(
        &mut self,
        _tls_manager: tls::ConfigManager,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        // TODO
        // Actually listen to file-events
        async fn run() {}

        Box::pin(run())
    }
}

/// The Dashboard-Entity for the File-Configurator
pub struct FileConfigurator {
    path: String,
}

impl FileConfigurator {
    /// Creates a new Empty Entity
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl DashboardEntity for FileConfigurator {
    fn get_type(&self) -> &str {
        "File"
    }
    fn get_content(&self) -> serde_json::Value {
        json!({
            "path": self.path,
        })
    }
}
