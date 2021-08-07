use tunneload::configurator::ConfigList;

use crate::{cmp_vec_contents, get_namespace};

pub async fn load_middleware() {
    let kube_client = kube::Client::try_default().await.unwrap();
    let test_namespace = get_namespace();

    let g_conf = tunneload::configurator::kubernetes::traefik_bindings::setup_general_configurator(
        kube_client,
        &test_namespace,
    );

    let plugin_list = ConfigList::new();

    let middlewares = g_conf.load_middlewares(&plugin_list).await;

    let strip_prefix_middleware = middlewares
        .iter()
        .find(|m| m.get_name() == "testing-middleware-strip-prefix")
        .expect("The Middleware should have been loaded");
    match strip_prefix_middleware.get_action() {
        tunneload::rules::Action::RemovePrefix(prefix) if prefix == "/test" => {}
        _ => assert!(false),
    };

    let headers_middleware = middlewares
        .iter()
        .find(|m| m.get_name() == "testing-middleware-headers")
        .expect("The Middleware should have been loaded");
    match headers_middleware.get_action() {
        tunneload::rules::Action::AddHeaders(headers) => assert!(cmp_vec_contents(
            &vec![
                ("test-header".to_owned(), "test-value".to_owned()),
                ("other-header".to_owned(), "other-value".to_owned()),
            ],
            headers,
        )),
        _ => assert!(false),
    };

    let headers_cors_middleware = middlewares
        .iter()
        .find(|m| m.get_name() == "testing-middleware-headers-cors")
        .expect("The Middleware should have been loaded");
    match headers_cors_middleware.get_action() {
        tunneload::rules::Action::Cors(opts) => assert_eq!(
            &tunneload::rules::CorsOpts {
                origins: vec![],
                max_age: None,
                credentials: false,
                methods: vec!["GET".to_owned()],
                headers: vec![],
            },
            opts
        ),
        _ => assert!(false),
    };

    let compress_middleware = middlewares
        .iter()
        .find(|m| m.get_name() == "testing-middleware-compress")
        .expect("The Middleware should have been loaded");
    match compress_middleware.get_action() {
        tunneload::rules::Action::Compress => {}
        _ => assert!(false),
    };
}
