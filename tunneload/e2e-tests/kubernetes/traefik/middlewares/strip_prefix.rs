use std::path::PathBuf;

use general::{Group, Name};
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

    let config_file = get_config_file("strip_prefix.yaml");
    let runner = kubectl::KubeCtlRunner::new(kubectl::Command::Apply {
        resource: kubectl::Resource::File(config_file),
    });

    runner
        .run()
        .await
        .expect("Setting up the kubernetes Test Environment");
}

async fn teardown() {
    let runner = kubectl::KubeCtlRunner::new(kubectl::Command::Delete {
        resource: kubectl::Resource::Specific("namespace".to_owned(), "testing".to_owned()),
    });

    runner
        .run()
        .await
        .expect("Tearing down kubernetes Test Environment");

    teardown_crds().await;
}

async fn strip_prefix() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = "testing";

    let g_conf = tunneload::configurator::kubernetes::traefik_bindings::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    let middlewares = g_conf.load_middlewares(&ConfigList::new()).await;

    let expected_name = Name::new(
        "testing-middleware-strip-prefix",
        Group::Kubernetes {
            namespace: "testing".to_owned(),
        },
    );

    let strip_prefix_middleware = middlewares
        .iter()
        .find(|m| m.get_name() == &expected_name)
        .expect("The Middleware should have been loaded");
    match strip_prefix_middleware.get_action() {
        rules::Action::RemovePrefix(prefix) if prefix == "/test" => {}
        _ => assert!(false),
    };
}

inventory::submit! {
    E2ETest::with_setup_teardown("K8S-Traefik-Middlewares-StripPrefix", setup, strip_prefix, teardown)
}
