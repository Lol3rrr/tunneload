use criterion::{black_box, criterion_group, criterion_main, Criterion};

use tunneload::http;

fn generate_headers<'a>(count: usize) -> http::Headers<'a> {
    let mut result = http::Headers::new();
    for i in 0..count {
        let key = format!("Key-{:03}", i);
        let value = format!("Value-{:03}", i);

        result.add(key, value);
    }
    result
}

fn generate_header_text(count: usize) -> String {
    let mut result = String::new();
    for i in 0..count {
        let key = format!("Key-{:03}", i);
        let value = format!("Value-{:03}", i);

        let line = format!("{}: {}\r\n", key, value);
        result.push_str(&line);
    }
    result
}

pub fn request_parsing(c: &mut Criterion) {
    let mut content = "GET /test HTTP/1.1\r\n".to_owned();
    content.push_str(&generate_header_text(20));
    content.push_str("\r\n");

    let content_bytes = content.as_bytes();

    c.bench_function("HTTP-Request-Stream-Parse-NoBody", |b| {
        b.iter(|| {
            let mut parser = http::streaming_parser::ReqParser::new_capacity(4096);
            parser.block_parse(black_box(content_bytes));
        })
    });

    let mut parser = http::streaming_parser::ReqParser::new_capacity(4096);
    parser.block_parse(black_box(content_bytes));
    c.bench_function("HTTP-Request-Stream-Finish-NoBody", |b| {
        b.iter(|| parser.finish())
    });

    let headers = generate_headers(20);
    let req = http::Request::new(
        "HTTP/1.1",
        http::Method::GET,
        "/path",
        headers.clone(),
        "some random body".as_bytes(),
    );
    c.bench_function("HTTP-Request-Serialize", |b| b.iter(|| req.serialize()));
}

pub fn response_parsing(c: &mut Criterion) {
    let resp_content = "HTTP/1.1 200 OK\r\nTest-1: Value-1\r\nTest-2: Value-2\r\n\r\n".as_bytes();
    let mut parser = http::streaming_parser::RespParser::new_capacity(4096);
    c.bench_function("HTTP-Response-Stream-Parse-NoBody", |b| {
        b.iter(|| {
            parser.block_parse(black_box(resp_content));
            parser.clear();
        })
    });

    let mut parser = http::streaming_parser::RespParser::new_capacity(1024);
    parser.block_parse(resp_content);
    c.bench_function("HTTP-Response-Stream-Finish-NoBody", |b| {
        b.iter(|| {
            parser.finish();
        })
    });

    let headers = generate_headers(20);
    let resp = http::Response::new(
        "HTTP/1.1",
        http::StatusCode::OK,
        headers,
        "Random Response Body".as_bytes().to_vec(),
    );
    c.bench_function("HTTP-Response-Serialize", |b| b.iter(|| resp.serialize()));
}

criterion_group!(benches, request_parsing, response_parsing);
criterion_main!(benches);
