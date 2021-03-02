use criterion::{black_box, BenchmarkId, Criterion};
use tunneload::http;

use crate::{generate_header_text, generate_headers};

pub fn parsing(c: &mut Criterion) {
    let mut req_parse_group = c.benchmark_group("HTTP-Request-Parser");
    for header_count in [2, 4, 8, 16, 32usize].iter() {
        let mut req_parse_content = "GET /test HTTP/1.1\r\n".to_owned();
        req_parse_content.push_str(&generate_header_text(*header_count));
        req_parse_content.push_str("\r\n");

        let req_parse_content_bytes = req_parse_content.as_bytes();

        req_parse_group.bench_function(BenchmarkId::from_parameter(header_count), |b| {
            let mut parser = http::streaming_parser::ReqParser::new_capacity(2048);
            b.iter(|| {
                parser.block_parse(black_box(req_parse_content_bytes));
                parser.clear();
            })
        });
    }
}

pub fn parse_finish(c: &mut Criterion) {
    let mut group = c.benchmark_group("HTTP-Request-Finish");
    for header_count in [2, 4, 8, 16, 32usize].iter() {
        let mut content = "GET /test HTTP/1.1\r\n".to_owned();
        content.push_str(&generate_header_text(*header_count));
        content.push_str("\r\n");
        let content_bytes = content.as_bytes();

        let mut parser = http::streaming_parser::ReqParser::new_capacity(2048);
        parser.block_parse(content_bytes);

        group.bench_function(BenchmarkId::from_parameter(header_count), |b| {
            b.iter(|| parser.finish())
        });
    }
}

pub fn serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("HTTP-Request-Serialize");
    for header_count in [2, 4, 8, 16, 32usize].iter() {
        let headers = generate_headers(*header_count);
        let req = http::Request::new(
            "HTTP/1.1",
            http::Method::GET,
            "/path",
            headers.clone(),
            "some random body".as_bytes(),
        );

        group.bench_function(BenchmarkId::from_parameter(header_count), |b| {
            b.iter(|| req.serialize())
        });
    }
}
