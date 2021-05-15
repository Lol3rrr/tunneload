use k8s_openapi::api::extensions::v1beta1::Ingress;
use kube::{
    api::{ListParams, Meta},
    Api,
};

use crate::configurator::{
    kubernetes::general::{Event, Watcher},
    kubernetes::ingress::parse::parse_rule,
    RuleList,
};

/// Listens to IngressRule Events and updates the Configuration
/// accordingly
pub async fn listen_rules(
    client: kube::Client,
    namespace: String,
    rules: RuleList,
    default_priority: u32,
) {
    let middleware_crds: Api<Ingress> = Api::namespaced(client.clone(), &namespace);

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
                match parse_rule(mid, default_priority) {
                    Ok(mut parsed) => {
                        for tmp in parsed.drain(..) {
                            log::info!("Updated Rule: {:?}", tmp);
                            rules.set_rule(tmp);
                        }
                    }
                    Err(e) => {
                        log::error!("Parsing-Rule: {:?}", e);
                    }
                };
            }
            Event::Removed(srv) => {
                let name = Meta::name(&srv);

                rules.remove_rule(name);
            }
            Event::Other => {}
        };
    }
}
