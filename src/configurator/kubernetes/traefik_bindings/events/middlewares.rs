use kube::{
    api::{ListParams, Meta},
    Api,
};

use crate::configurator::{
    kubernetes::{
        general::{Event, Watcher},
        traefik_bindings::{
            middleware::{Config, Middleware},
            parse::parse_middleware,
        },
    },
    ConfigItem, MiddlewareList,
};

pub async fn listen_middlewares(
    client: kube::Client,
    namespace: String,
    middlewares: MiddlewareList,
) {
    let middleware_crds: Api<Middleware> = Api::namespaced(client.clone(), &namespace);

    let lp = ListParams::default();

    let mut stream = match Watcher::from_api(middleware_crds, Some(lp)).await {
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
            Event::Updated(mid) => {
                let metadata = mid.metadata;
                if let Some(raw_annotations) = metadata.annotations {
                    let last_applied = raw_annotations
                        .get("kubectl.kubernetes.io/last-applied-configuration")
                        .unwrap();

                    let current_config: Config = serde_json::from_str(last_applied).unwrap();

                    let mut res_middlewares =
                        parse_middleware(Some(client.clone()), Some(&namespace), current_config)
                            .await;

                    for tmp in res_middlewares.drain(..) {
                        log::info!("Updated Middleware: {}", tmp.name());

                        middlewares.set_middleware(tmp);
                    }
                }
            }
            Event::Removed(srv) => {
                let name = Meta::name(&srv);

                log::info!("Deleting Middleware: {}", name);

                middlewares.remove_middleware(&name);
            }
            Event::Other => {}
        };
    }
}
