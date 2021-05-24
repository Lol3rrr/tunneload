use stream_httparse::{header::HeaderValue, Headers, Request, Response};
use tunneload::plugins::MiddlewarePlugin;

#[test]
fn apply_set_header_not_set() {
    let data = std::fs::read("./tests/plugins/set_header.wasm").unwrap();

    let plugin = MiddlewarePlugin::new(&data, "".to_owned()).unwrap();

    let mut request = Request::new(
        "HTTP/1.1",
        stream_httparse::Method::GET,
        "/test/",
        Headers::new(),
        &[],
    );
    plugin.apply_req(&mut request).unwrap();

    assert_eq!(None, request.headers().get("result-key"));
}

#[test]
fn apply_set_header_set_false_value() {
    let data = std::fs::read("./tests/plugins/set_header.wasm").unwrap();

    let plugin = MiddlewarePlugin::new(&data, "".to_owned()).unwrap();

    let mut headers = Headers::new();
    headers.set("test-key", "random-value");
    let mut request = Request::new(
        "HTTP/1.1",
        stream_httparse::Method::GET,
        "/test/",
        headers,
        &[],
    );
    let apply_result = plugin.apply_req(&mut request);

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

    let plugin = MiddlewarePlugin::new(&data, "".to_owned()).unwrap();

    let mut headers = Headers::new();
    headers.set("test-key", "specific-value");
    let mut request = Request::new(
        "HTTP/1.1",
        stream_httparse::Method::GET,
        "/test/",
        headers,
        &[],
    );
    plugin.apply_req(&mut request).unwrap();

    let result_header = request.headers().get("result-key");
    assert!(result_header.is_some());
    assert_eq!(
        Some(&HeaderValue::Str("result-true".to_owned())),
        result_header
    );
}
