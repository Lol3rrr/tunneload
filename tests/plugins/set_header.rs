use stream_httparse::{header::HeaderValue, Headers, Request, Response};
use tunneload::plugins::{ActionPluginInstance, Plugin};

#[test]
fn apply_set_header_not_set() {
    let data = std::fs::read("./tests/plugins/set_header.wasm").unwrap();

    let plugin = Plugin::new("test_name".to_owned(), &data).unwrap();
    let instance: ActionPluginInstance = plugin.create_instance("".to_owned()).unwrap();

    let mut request = Request::new(
        "HTTP/1.1",
        stream_httparse::Method::GET,
        "/test/",
        Headers::new(),
        &[],
    );
    assert_eq!(Ok(()), instance.apply_req(&mut request));

    assert_eq!(None, request.headers().get("result-key"));
}

#[test]
fn apply_set_header_set_false_value() {
    let data = std::fs::read("./tests/plugins/set_header.wasm").unwrap();

    let plugin = Plugin::new("test_name".to_owned(), &data).unwrap();
    let instance: ActionPluginInstance = plugin.create_instance("".to_owned()).unwrap();

    let mut headers = Headers::new();
    headers.set("test-key", "random-value");
    let mut request = Request::new(
        "HTTP/1.1",
        stream_httparse::Method::GET,
        "/test/",
        headers,
        &[],
    );
    let apply_result = instance.apply_req(&mut request);

    assert!(apply_result.is_err());
    assert_eq!(
        Err(Response::new(
            "HTTP/1.1",
            stream_httparse::StatusCode::InternalServerError,
            Headers::new(),
            Vec::new()
        )),
        apply_result
    );
}

#[test]
fn apply_set_header_set_right_value() {
    let data = std::fs::read("./tests/plugins/set_header.wasm").unwrap();

    let plugin = Plugin::new("test_name".to_owned(), &data).unwrap();
    let instance: ActionPluginInstance = plugin.create_instance("".to_owned()).unwrap();

    let mut headers = Headers::new();
    headers.set("test-key", "specific-value");
    let mut request = Request::new(
        "HTTP/1.1",
        stream_httparse::Method::GET,
        "/test/",
        headers,
        &[],
    );
    instance.apply_req(&mut request).unwrap();

    let result_header = request.headers().get("result-key");
    assert!(result_header.is_some());
    assert_eq!(
        Some(&HeaderValue::Str("result-true".to_owned())),
        result_header
    );
}
