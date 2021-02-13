use crate::rules::Rule;
use crate::rules::WriteManager;

use crate::kubernetes::traefik_bindings;

use kube::Client;

use log::debug;

pub struct Manager {
    client: Client,
}

impl Manager {
    pub async fn new() -> Self {
        // Read the environment to find config for kube client.
        // Note that this tries an in-cluster configuration first,
        // then falls back on a kubeconfig file.
        let client = Client::try_default().await.unwrap();

        Self { client }
    }

    async fn get_rules(&self, namespace: &str) -> Vec<Rule> {
        let middlewares = traefik_bindings::load_middlewares(self.client.clone(), namespace).await;
        traefik_bindings::load_routes(self.client.clone(), namespace, &middlewares).await
    }

    async fn update_rules(&self, writer: &WriteManager) {
        debug!("Updating Rules");
        let rules = self.get_rules("default").await;
        writer.add_rules(rules);

        debug!("Updated Rules");
    }

    pub async fn update_loop(self, writer: WriteManager, wait_time: std::time::Duration) {
        loop {
            self.update_rules(&writer).await;

            tokio::time::sleep(wait_time).await;
        }
    }
}
