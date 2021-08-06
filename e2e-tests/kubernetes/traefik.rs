use tunneload::configurator::ConfigList;

use crate::get_namespace;

pub async fn load_middleware() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = get_namespace();

    let g_conf = tunneload::configurator::kubernetes::traefik_bindings::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    let plugin_list = ConfigList::new();

    let middlewares = g_conf.load_middlewares(&plugin_list).await;

    let loaded_middleware = middlewares
        .into_iter()
        .find(|m| m.get_name() == "testing-middleware-strip-prefix")
        .expect("The Middleware should have been loaded");

    match loaded_middleware.get_action() {
        tunneload::rules::Action::RemovePrefix(prefix) if prefix == "/test" => {}
        _ => assert!(false),
    };
}
