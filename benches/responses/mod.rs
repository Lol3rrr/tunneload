use criterion::{black_box, BenchmarkId, Criterion};
use tunneload::http;

use crate::{generate_header_text, generate_headers};

pub fn parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("HTTP-Response-Parser");
    for header_count in [2, 4, 8, 16, 32usize].iter() {
        let mut content = "HTTP/1.1 200 OK\r\n".to_owned();
        content.push_str(&generate_header_text(*header_count));
        content.push_str("\r\n");

        let content_bytes = content.as_bytes();

        group.bench_function(BenchmarkId::from_parameter(header_count), |b| {
            let mut parser = http::streaming_parser::RespParser::new_capacity(4096);
            b.iter(|| {
                parser.block_parse(black_box(content_bytes));
                parser.clear();
            })
        });
    }
}

pub fn parse_finish(c: &mut Criterion) {
    let mut group = c.benchmark_group("HTTP-Response-Finish");
    for header_count in [2, 4, 8, 16, 32usize].iter() {
        let mut content = "HTTP/1.1 200 OK\r\n".to_owned();
        content.push_str(&generate_header_text(*header_count));
        content.push_str("\r\n");

        let content_bytes = content.as_bytes();

        group.bench_function(BenchmarkId::from_parameter(header_count), |b| {
            let mut parser = http::streaming_parser::RespParser::new_capacity(4096);
            parser.block_parse(black_box(content_bytes));
            b.iter(|| {
                parser.finish();
            })
        });
    }
}

pub fn serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("HTTP-Response-Serialize");
    for header_count in [2, 4, 8, 16, 32usize].iter() {
        let headers = generate_headers(*header_count);
        let resp = http::Response::new(
            "HTTP/1.1",
            http::StatusCode::OK,
            headers,
            "Random Response Body".as_bytes().to_vec(),
        );

        group.bench_function(BenchmarkId::from_parameter(header_count), |b| {
            b.iter(|| {
                resp.serialize();
            })
        });
    }
}
