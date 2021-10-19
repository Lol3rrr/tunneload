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

    let config_file = get_config_file("headers.yaml");
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

async fn headers() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = "testing";

    let g_conf = tunneload::configurator::kubernetes::general::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    let middlewares = g_conf.load_middlewares(&ConfigList::new()).await;

    let headers_middleware = match middlewares
        .iter()
        .find(|m| m.get_name() == "testing-middleware-headers")
    {
        Some(m) => m,
        None => {
            panic!("Loaded middlewares did not contain one with the name: \"testing-middleware-headers\", got: {:?}", middlewares);
        }
    };

    match headers_middleware.get_action() {
        rules::Action::AddHeaders(headers) => assert!(cmp_vec_contents(
            &vec![
                ("test-header".to_owned(), "test-value".to_owned()),
                ("other-header".to_owned(), "other-value".to_owned()),
            ],
            headers,
        )),
        a => panic!("Expected AddHeades Action but got: {:?}", a),
    };
}

async fn headers_cors() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = "testing";

    let g_conf = tunneload::configurator::kubernetes::general::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    let middlewares = g_conf.load_middlewares(&ConfigList::new()).await;

    let headers_cors_middleware = match middlewares
        .iter()
        .find(|m| m.get_name() == "testing-middleware-headers-cors") {
            Some(m) => m,
            None => panic!("Loaded middlewares did not contain one with the name: \"testing-middleware-headers-cors\", got: {:?}", middlewares),
        };
    match headers_cors_middleware.get_action() {
        rules::Action::Cors(opts) => assert_eq!(
            &rules::CorsOpts {
                origins: vec![],
                max_age: None,
                credentials: false,
                methods: vec!["GET".to_owned()],
                headers: vec![],
            },
            opts
        ),
        a => panic!("Expected CORS Action but got: {:?}", a),
    };
}

inventory::submit! {
    E2ETest::with_setup_teardown("K8S-Traefik-Middlewares-Headers", setup, headers, teardown)
}

inventory::submit! {
    E2ETest::with_setup_teardown("K8S-Traefik-Middlewares-Headers-Cors", setup, headers_cors, teardown)
}
