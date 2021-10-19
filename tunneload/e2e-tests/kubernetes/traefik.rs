use std::path::PathBuf;

use crate::{current_source_dir, get_namespace, tests::E2ETest};

mod middlewares;
mod rules;

fn get_config_file(name: &str) -> PathBuf {
    let mut current = current_source_dir!();
    current.push("config/");
    current.push(name);

    current
}

async fn setup_crds() {
    let config_file = get_config_file("traefik-crds.yaml");
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

async fn teardown_crds() {
    let config_file = get_config_file("traefik-crds.yaml");
    let config_file_path = config_file.to_str().unwrap();

    let mut kubectl_handle = tokio::process::Command::new("kubectl")
        .arg("delete")
        .arg("-f")
        .arg(config_file_path)
        .spawn()
        .expect("Running Kubectl to setup env for tests");

    kubectl_handle
        .wait()
        .await
        .expect("Could not setup Environment using Kubectl");
}
