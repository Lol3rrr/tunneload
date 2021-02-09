use criterion::{black_box, criterion_group, criterion_main, Criterion};

use tunneload::http;

pub fn criterion_benchmark(c: &mut Criterion) {
    let content = "GET /test HTTP/1.1\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\n".as_bytes();
    c.bench_function("HTTP-Parse-NoBody", |b| {
        b.iter(|| http::Request::parse(black_box(content)))
    });

    let req = http::Request::new(
        "HTTP/1.1",
        http::Method::GET,
        "/path",
        vec![
            http::Header::new("key-1", "value-1"),
            http::Header::new("key-2", "value-2"),
        ],
        "some random body".as_bytes(),
    );
    c.bench_function("HTTP-Serialize-Request", |b| b.iter(|| req.serialize()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
