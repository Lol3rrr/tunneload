use std::path::PathBuf;

use tunneload::configurator::{parser::GeneralConfigurator, ConfigItem, ConfigList};

use crate::tests::{current_source_dir, E2ETest};

use super::{setup_crds, teardown_crds};

fn get_config_file(name: &str) -> PathBuf {
    let mut current = current_source_dir!();
    current.push("config/");
    current.push(name);

    current
}

async fn setup_minimal() {
    setup_crds().await;

    let config_file = get_config_file("minimal.yaml");
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

async fn teardown_minimal() {
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

async fn minimal() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = "testing";

    let g_conf = tunneload::configurator::kubernetes::general::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    let middlewares = ConfigList::new();
    let services = ConfigList::new();

    // Load the Rules, without a cert_queue to stop it from marking the Rule-TLS as Generate
    let rule_list = g_conf.load_rules(&middlewares, &services, None).await;

    let minimal_rule = rule_list
        .iter()
        .find(|r| r.name() == "testing-rule-minimal")
        .expect("The Rule should have been loaded");

    assert_eq!(
        &rules::Matcher::Domain("example.com".to_owned()),
        minimal_rule.matcher()
    );
    assert_eq!(1, minimal_rule.priority());
    assert_eq!(
        std::sync::Arc::new(rules::Service::new(
            "testing-service-minimal@testing",
            Vec::new()
        )),
        minimal_rule.service()
    );
    assert_eq!(&rules::RuleTLS::None, minimal_rule.tls());
}

inventory::submit! {
    E2ETest::with_setup_teardown("K8S-Traefik-Rules-minimal", setup_minimal, minimal, teardown_minimal)
}
