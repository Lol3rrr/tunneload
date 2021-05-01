use kube::{
    api::{ListParams, Meta},
    Api,
};

use crate::configurator::{
    kubernetes::{
        general::{Event, Watcher},
        traefik_bindings::{
            ingressroute::{Config, IngressRoute},
            parse::parse_rule,
        },
    },
    MiddlewareList, RuleList, ServiceList,
};

use log::error;

pub async fn listen_rules(
    client: kube::Client,
    namespace: String,
    middlewares: MiddlewareList,
    services: ServiceList,
    rules: RuleList,
) {
    let middleware_crds: Api<IngressRoute> = Api::namespaced(client.clone(), &namespace);

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

                    match parse_rule(current_config, &middlewares, &services) {
                        Ok(r) => {
                            rules.set_rule(r);
                        }
                        Err(e) => {
                            error!("Unknown Rule('{:?}'): {:?}", last_applied, e);
                        }
                    };
                }
            }
            Event::Removed(srv) => {
                let name = Meta::name(&srv);

                rules.remove_rule(name);
            }
            Event::Other => {}
        };
    }
}
