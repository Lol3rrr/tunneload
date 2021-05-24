use stream_httparse::{Headers, Request};
use tunneload::plugins::MiddlewarePlugin;

#[test]
fn load_middleware() {
    let data = std::fs::read("./tests/plugins/strip_prefix.wasm").unwrap();

    assert!(MiddlewarePlugin::new(&data, "/test".to_owned()).is_some());
}

#[test]
fn apply_strip_prefix() {
    env_logger::init();

    let data = std::fs::read("./tests/plugins/strip_prefix.wasm").unwrap();

    let plugin = MiddlewarePlugin::new(&data, "/test".to_owned()).unwrap();

    let mut request = Request::new(
        "HTTP/1.1",
        stream_httparse::Method::GET,
        "/test/",
        Headers::new(),
        &[],
    );
    plugin.apply_req(&mut request).unwrap();

    assert_eq!("/", request.path());
}
