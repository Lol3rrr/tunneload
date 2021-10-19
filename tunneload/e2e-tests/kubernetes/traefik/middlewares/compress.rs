use std::path::PathBuf;

use tunneload::configurator::{parser::GeneralConfigurator, ConfigList};

use crate::{
    cmp_vec_contents, current_source_dir,
    kubernetes::{
        kubectl,
        traefik::{setup_crds, teardown_crds},
    },
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

    let config_file = get_config_file("compress.yaml");
    let runner = kubectl::KubeCtlRunner::new(kubectl::Command::Apply {
        resource: kubectl::Resource::File(config_file),
    });

    runner
        .run()
        .await
        .expect("Setting up Kubernetes Test Environment");
}

async fn teardown() {
    let runner = kubectl::KubeCtlRunner::new(kubectl::Command::Delete {
        resource: kubectl::Resource::Specific("namespace".to_owned(), "testing".to_string()),
    });

    runner
        .run()
        .await
        .expect("Tearing down the Kubernetes Setup for testing");

    teardown_crds().await;
}

async fn compress() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = "testing";

    let g_conf = tunneload::configurator::kubernetes::general::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    let middlewares = g_conf.load_middlewares(&ConfigList::new()).await;

    let compress_middleware = middlewares
        .iter()
        .find(|m| m.get_name() == "testing-middleware-compress")
        .expect("The Middleware should have been loaded");
    match compress_middleware.get_action() {
        rules::Action::Compress => {}
        _ => assert!(false),
    };
}

inventory::submit! {
    E2ETest::with_setup_teardown("K8S-Traefik-Middlewares-Compress", setup, compress, teardown)
}
