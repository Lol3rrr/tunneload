use async_trait::async_trait;
use futures::FutureExt;

use crate::{
    configurator::{
        files::Config,
        parser::{self, EventEmitter, EventFuture, RawMiddlewareConfig, RawRuleConfig},
    },
    util::files::events,
};

/// The Event-Emitter for the File-Configuration
pub struct FileEvents {
    path: String,
}

impl FileEvents {
    /// Creates a new Instace of the Event-Emitter that listens for
    /// Events for the given Path
    pub fn new(path: String) -> Self {
        Self { path }
    }

    fn parse_middleware(tmp_middle: &serde_json::Value) -> Option<Vec<RawMiddlewareConfig>> {
        let name = tmp_middle.get("name")?;
        let name = name.as_str()?;

        let tmp_obj = tmp_middle.as_object()?;

        let cap = tmp_obj.keys().len().saturating_sub(1);
        let mut result = Vec::with_capacity(cap);
        for key in tmp_obj.keys() {
            if key == "name" {
                continue;
            }

            let value = tmp_obj.get(key).unwrap();

            result.push(RawMiddlewareConfig {
                name: name.to_string(),
                action_name: key.to_string(),
                config: value.to_owned(),
            });
        }

        Some(result)
    }

    async fn middleware_events(
        path: String,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawMiddlewareConfig>>,
    ) {
        let watcher = match events::CustomWatcher::new(path) {
            Some(w) => w,
            None => {
                tracing::error!("Failed to create Middleware-File-Watcher");
                return;
            }
        };

        for path in watcher {
            let content = match std::fs::read(&path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Reading File: {:?}", e);
                    continue;
                }
            };

            let deserialized: Config = match serde_yaml::from_slice(&content) {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!("Deserializing Config: {:?}", e);
                    continue;
                }
            };

            let middlewares = match deserialized.middleware {
                Some(m) => m,
                None => continue,
            };

            for tmp in middlewares.iter() {
                let mut parsed = match Self::parse_middleware(tmp) {
                    Some(p) => p,
                    None => {
                        tracing::error!("Parsing Middlewares");
                        continue;
                    }
                };

                for p in parsed.drain(..) {
                    if let Err(e) = sender.send(parser::Event::Update(p)) {
                        tracing::error!("Sending Event: {:?}", e);
                        return;
                    }
                }
            }
        }
    }

    async fn rule_events(
        path: String,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawRuleConfig>>,
    ) {
        let watcher = match events::CustomWatcher::new(path) {
            Some(w) => w,
            None => {
                tracing::error!("Failed to create Rule-File-Watcher");
                return;
            }
        };

        for path in watcher {
            let content = match std::fs::read(&path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Reading File: {:?}", e);
                    continue;
                }
            };

            let deserialized: Config = match serde_yaml::from_slice(&content) {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!("Parsing Config: {:?}", e);
                    continue;
                }
            };

            let routes = match deserialized.routes {
                Some(r) => r,
                None => continue,
            };

            for tmp in routes {
                let value = serde_json::to_value(tmp).unwrap();

                if let Err(e) = sender.send(parser::Event::Update(RawRuleConfig { config: value }))
                {
                    tracing::error!("Sending Event: {:?}", e);
                    return;
                }
            }
        }
    }
}

#[async_trait]
impl EventEmitter for FileEvents {
    async fn middleware_listener(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawMiddlewareConfig>>,
    ) -> Option<EventFuture> {
        async fn run(
            path: String,
            sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawMiddlewareConfig>>,
        ) {
            tokio::task::spawn_blocking(move || {
                futures::executor::block_on(FileEvents::middleware_events(path, sender));
            });
        }

        Some(run(self.path.clone(), sender).boxed())
    }

    async fn rule_listener(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawRuleConfig>>,
    ) -> Option<EventFuture> {
        async fn run(
            path: String,
            sender: tokio::sync::mpsc::UnboundedSender<parser::Event<RawRuleConfig>>,
        ) {
            tokio::task::spawn_blocking(move || {
                futures::executor::block_on(FileEvents::rule_events(path, sender));
            });
        }

        Some(run(self.path.clone(), sender).boxed())
    }
}
