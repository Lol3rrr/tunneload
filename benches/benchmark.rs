use criterion::{black_box, criterion_group, criterion_main, Criterion};

use tunneload::http;

pub fn criterion_benchmark(c: &mut Criterion) {
    let content = "GET /test HTTP/1.1\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\n".as_bytes();
    c.bench_function("HTTP-Request-Parse-NoBody", |b| {
        b.iter(|| http::Request::parse(black_box(content)))
    });

    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Key-1".to_owned(), "Value-1".to_owned());
    headers.insert("Key-2".to_owned(), "Value-2".to_owned());
    let req = http::Request::new(
        "HTTP/1.1",
        http::Method::GET,
        "/path",
        headers.clone(),
        "some random body".as_bytes(),
    );
    c.bench_function("HTTP-Request-Serialize", |b| b.iter(|| req.serialize()));

    let resp_content = "HTTP/1.1 200 OK\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\n".as_bytes();
    c.bench_function("HTTP-Response-Parse-NoBody", |b| {
        b.iter(|| http::Response::parse(black_box(resp_content)))
    });

    let resp = http::Response::new(
        "HTTP/1.1",
        http::StatusCode::OK,
        headers,
        "Random Response Body".as_bytes(),
    );
    c.bench_function("HTTP-Response-Serialize", |b| b.iter(|| resp.serialize()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
