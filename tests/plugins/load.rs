use stream_httparse::{Headers, Request};
use tunneload::plugins::{ActionPluginInstance, Plugin};

#[test]
fn load_middleware() {
    let data = std::fs::read("./tests/plugins/strip_prefix.wasm").unwrap();

    assert!(Plugin::new("test_name".to_owned(), &data).is_some());
}

#[test]
fn create_instance() {
    let data = std::fs::read("./tests/plugins/strip_prefix.wasm").unwrap();

    let plugin = Plugin::new("test_name".to_owned(), &data).unwrap();
    assert!(plugin
        .create_instance::<ActionPluginInstance>("/test".to_owned())
        .is_some());
}

#[test]
fn apply_strip_prefix() {
    let data = std::fs::read("./tests/plugins/strip_prefix.wasm").unwrap();

    let plugin = Plugin::new("test_name".to_owned(), &data).unwrap();
    let instance: ActionPluginInstance = plugin.create_instance("/test".to_owned()).unwrap();

    let mut request = Request::new(
        "HTTP/1.1",
        stream_httparse::Method::GET,
        "/test/",
        Headers::new(),
        &[],
    );
    instance.apply_req(&mut request).unwrap();

    assert_eq!("/", request.path());
}
