use crate::get_namespace;

mod service;

pub async fn load_service() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = get_namespace();

    let g_conf = tunneload::configurator::kubernetes::general::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    service::load(&g_conf).await;
}
