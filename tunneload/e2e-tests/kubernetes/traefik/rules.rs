use std::path::PathBuf;

use general::{Group, Name};
use tunneload::configurator::{parser::GeneralConfigurator, ConfigItem, ConfigList};

use crate::{
    kubernetes::kubectl,
    tests::{current_source_dir, E2ETest},
};

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

    let runner = kubectl::KubeCtlRunner::new(kubectl::Command::Apply {
        resource: kubectl::Resource::File(config_file),
    });

    runner
        .run()
        .await
        .expect("Setting up Kubernetes Test Environment");
}

async fn teardown_minimal() {
    let runner = kubectl::KubeCtlRunner::new(kubectl::Command::Delete {
        resource: kubectl::Resource::Specific("namespace".to_owned(), "testing".to_owned()),
    });

    runner
        .run()
        .await
        .expect("Tearing down Kubernetes Environment");

    teardown_crds().await;
}

async fn minimal() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = "testing";

    let g_conf = tunneload::configurator::kubernetes::traefik_bindings::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    let middlewares = ConfigList::new();
    let services = ConfigList::new();

    // Load the Rules, without a cert_queue to stop it from marking the Rule-TLS as Generate
    let rule_list = g_conf.load_rules(&middlewares, &services, None).await;

    let expected_name = Name::new(
        "testing-rule-minimal",
        Group::Kubernetes {
            namespace: "testing".to_owned(),
        },
    );

    let minimal_rule = rule_list
        .iter()
        .find(|r| r.name() == &expected_name)
        .expect("The Rule should have been loaded");

    assert_eq!(
        &rules::Matcher::Domain("example.com".to_owned()),
        minimal_rule.matcher()
    );
    assert_eq!(1, minimal_rule.priority());
    assert_eq!(
        std::sync::Arc::new(rules::Service::new(
            Name::new(
                "testing-service-minimal",
                Group::Kubernetes {
                    namespace: "testing".to_owned()
                }
            ),
            Vec::new()
        )),
        minimal_rule.service()
    );
    assert_eq!(&rules::RuleTLS::None, minimal_rule.tls());
}

inventory::submit! {
    E2ETest::with_setup_teardown("K8S-Traefik-Rules-minimal", setup_minimal, minimal, teardown_minimal)
}
