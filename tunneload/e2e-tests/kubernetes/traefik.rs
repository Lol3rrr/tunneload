use crate::{get_namespace, tests::E2ETest};

mod middlewares;
mod rules;

pub async fn load_middleware() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = get_namespace();

    let g_conf = tunneload::configurator::kubernetes::traefik_bindings::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    middlewares::stip_prefix(&g_conf).await;
    middlewares::headers(&g_conf).await;
    middlewares::headers_cors(&g_conf).await;
    middlewares::compress(&g_conf).await;
}

pub async fn load_rules() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = get_namespace();

    let g_conf = tunneload::configurator::kubernetes::traefik_bindings::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    rules::minimal(&g_conf).await;
}

inventory::submit! {
    E2ETest::only_test("K8S-Traefik-LoadMiddlewares", load_middleware)
}
inventory::submit! {
    E2ETest::only_test("K8S-Traefik-LoadRules", load_rules)
}
