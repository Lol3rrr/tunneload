use criterion::{black_box, criterion_group, criterion_main, Criterion};

use tunneload::http;

pub fn criterion_benchmark(c: &mut Criterion) {
    let content = "GET /test HTTP/1.1\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\n".as_bytes();
    c.bench_function("HTTP-Parse-NoBody", |b| {
        b.iter(|| http::Request::parse(black_box(content)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
