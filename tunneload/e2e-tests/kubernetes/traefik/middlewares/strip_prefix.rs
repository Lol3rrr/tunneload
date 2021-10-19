use std::path::PathBuf;

use tunneload::configurator::{parser::GeneralConfigurator, ConfigList};

use crate::{
    cmp_vec_contents, current_source_dir,
    kubernetes::traefik::{setup_crds, teardown_crds},
    tests::E2ETest,
};

fn get_config_file(name: &str) -> PathBuf {
    let mut current = current_source_dir!();
    current.push("config/");
    current.push(name);

    current
}

async fn setup() {
    setup_crds().await;

    let config_file = get_config_file("strip_prefix.yaml");
    let config_file_path = config_file.to_str().unwrap();

    let mut kubectl_handle = tokio::process::Command::new("kubectl")
        .arg("apply")
        .arg("-f")
        .arg(config_file_path)
        .spawn()
        .expect("Running Kubectl to setup env for tests");

    kubectl_handle
        .wait()
        .await
        .expect("Could not setup Environment using Kubectl");
}

async fn teardown() {
    let mut kubectl_handle = tokio::process::Command::new("kubectl")
        .arg("delete")
        .arg("namespace")
        .arg("testing")
        .spawn()
        .expect("Running kubectl to teardown env for tests");

    kubectl_handle
        .wait()
        .await
        .expect("Could not teardown Environment using kubectl");

    teardown_crds().await;
}

async fn strip_prefix() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = "testing";

    let g_conf = tunneload::configurator::kubernetes::general::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    let middlewares = g_conf.load_middlewares(&ConfigList::new()).await;

    let strip_prefix_middleware = middlewares
        .iter()
        .find(|m| m.get_name() == "testing-middleware-strip-prefix")
        .expect("The Middleware should have been loaded");
    match strip_prefix_middleware.get_action() {
        rules::Action::RemovePrefix(prefix) if prefix == "/test" => {}
        _ => assert!(false),
    };
}

inventory::submit! {
    E2ETest::with_setup_teardown("K8S-Traefik-Middlewares-StripPrefix", setup, strip_prefix, teardown)
}
