use std::path::PathBuf;

use general_traits::ConfigItem;
use tunneload::configurator::parser::GeneralConfigurator;

use crate::tests::{current_source_dir, E2ETest};

fn get_config_file(name: &str) -> PathBuf {
    let mut current = current_source_dir!();
    current.push("config/");
    current.push(name);

    current
}

async fn setup_simple() {
    let config_file = get_config_file("simple.yaml");

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
        .expect("Could not setup Enviroment using kubectl");
}

async fn teardown_simple() {
    let config_file = get_config_file("simple.yaml");

    let config_file_path = config_file.to_str().unwrap();

    let mut kubectl_handle = tokio::process::Command::new("kubectl")
        .arg("delete")
        .arg("namespace")
        .arg("testing")
        .spawn()
        .expect("Running Kubectl to setup env for tests");

    kubectl_handle
        .wait()
        .await
        .expect("Could not setup Enviroment using kubectl");
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
