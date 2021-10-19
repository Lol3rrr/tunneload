use std::path::PathBuf;

use general_traits::ConfigItem;
use tunneload::configurator::parser::GeneralConfigurator;

use crate::{
    kubernetes::kubectl,
    tests::{current_source_dir, E2ETest},
};

fn get_config_file(name: &str) -> PathBuf {
    let mut current = current_source_dir!();
    current.push("config/");
    current.push(name);

    current
}

async fn setup_simple() {
    let config_file = get_config_file("simple.yaml");

    let runner = kubectl::KubeCtlRunner::new(
        kubectl::Command::Apply,
        kubectl::Resource::File(config_file),
    );

    runner
        .run()
        .await
        .expect("Setting up the Kubernetes Test Enviroment");
}

async fn teardown_simple() {
    let runner = kubectl::KubeCtlRunner::new(
        kubectl::Command::Delete,
        kubectl::Resource::Namespace("testing".to_owned()),
    );

    runner
        .run()
        .await
        .expect("Tearing down Kubernetes Test Environment");
}

async fn simple() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = "testing";

    let conf = tunneload::configurator::kubernetes::general::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    let result = conf.load_services().await;

    let is_contained = result
        .iter()
        .find(|s| s.name() == "test-service@testing")
        .is_some();

    if !is_contained {
        panic!(
            "Expected: {:?} to contain a service with the Name: {:?}",
            result, "test-service@testing"
        );
    }

    assert!(true);
}

inventory::submit! {
    E2ETest::with_setup_teardown("K8S-General-Service-Simple", setup_simple, simple, teardown_simple)
}
