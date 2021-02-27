use crate::http::{HeaderValue, Request, Response};

use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;

#[cfg(test)]
use crate::http::{Headers, Method, StatusCode};

fn is_compression_enabled(tokens: &str) -> bool {
    for token in tokens.split(", ") {
        if token == "gzip" {
            return true;
        }
    }

    false
}

pub fn apply_req(req: &Request<'_>, resp: &mut Response<'_>) {
    if resp.headers().get("Content-Encoding").is_some() {
        return;
    }
    if resp.headers().get("Transfer-Encoding").is_some() {
        return;
    }
    match req.headers().get("Accept-Encoding") {
        Some(value) => {
            let tmp: &str = match value {
                HeaderValue::Str(ref value) => &value,
                HeaderValue::StrRef(ref value) => value,
                _ => {
                    return;
                }
            };

            if !is_compression_enabled(tmp) {
                return;
            }
        }
        None => {
            return;
        }
    };

    let mut e = GzEncoder::new(Vec::with_capacity(resp.body().len()), Compression::fast());
    e.write_all(&resp.body).unwrap();

    let n_body = e.finish().unwrap();
    let body_length = n_body.len();
    resp.body = n_body;
    resp.add_header("content-encoding", "gzip");
    resp.add_header("content-length", body_length);
}

#[test]
fn apply_valid() {
    let mut req_headers = Headers::new();
    req_headers.add("Accept-Encoding", "gzip, deflate, br");
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/some/path",
        req_headers,
        "".as_bytes(),
    );

    let resp_body = "test".as_bytes().to_vec();
    let mut resp_headers = Headers::new();
    resp_headers.add("Content-Length", resp_body.len());
    let mut resp = Response::new("HTTP/1.1", StatusCode::OK, resp_headers, resp_body);

    apply_req(&req, &mut resp);

    let n_body = [
        31, 139, 8, 0, 0, 0, 0, 0, 4, 255, 43, 73, 45, 46, 1, 0, 12, 126, 127, 216, 4, 0, 0, 0,
    ];
    assert_eq!(n_body, resp.body());
    assert_eq!(
        Some(&HeaderValue::StrRef("gzip")),
        resp.headers().get("content-encoding")
    );
    assert_eq!(
        Some(&HeaderValue::NumberUsize(n_body.len())),
        resp.headers.get("content-length")
    );
}

#[test]
fn apply_valid_lowercase_header() {
    let mut req_headers = Headers::new();
    req_headers.add("accept-encoding", "gzip, deflate, br");
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/some/path",
        req_headers,
        "".as_bytes(),
    );

    let resp_body = "test".as_bytes().to_vec();
    let mut resp_headers = Headers::new();
    resp_headers.add("Content-Length", resp_body.len());
    let mut resp = Response::new("HTTP/1.1", StatusCode::OK, resp_headers, resp_body);

    apply_req(&req, &mut resp);

    let n_body = [
        31, 139, 8, 0, 0, 0, 0, 0, 4, 255, 43, 73, 45, 46, 1, 0, 12, 126, 127, 216, 4, 0, 0, 0,
    ];
    assert_eq!(n_body, resp.body());
    assert_eq!(
        Some(&HeaderValue::StrRef("gzip")),
        resp.headers().get("content-encoding")
    );
    assert_eq!(
        Some(&HeaderValue::NumberUsize(n_body.len())),
        resp.headers.get("content-length")
    );
}

#[test]
fn apply_no_gzip_not_accepted() {
    let mut req_headers = Headers::new();
    req_headers.add("Accept-Encoding", "deflate, br");
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/some/path",
        req_headers,
        "".as_bytes(),
    );

    let resp_body = "test".as_bytes().to_vec();
    let mut resp_headers = Headers::new();
    resp_headers.add("Content-Length", resp_body.len());
    let mut resp = Response::new("HTTP/1.1", StatusCode::OK, resp_headers, resp_body.clone());

    apply_req(&req, &mut resp);
    assert_eq!(&resp_body, resp.body());
    assert_eq!(None, resp.headers().get("content-encoding"));
    assert_eq!(
        Some(&HeaderValue::NumberUsize(resp_body.len())),
        resp.headers.get("content-length")
    );
}

#[test]
fn apply_no_accept_encoding_header() {
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/some/path",
        Headers::new(),
        "".as_bytes(),
    );

    let resp_body = "test".as_bytes().to_vec();
    let mut resp_headers = Headers::new();
    resp_headers.add("Content-Length", resp_body.len());
    let mut resp = Response::new("HTTP/1.1", StatusCode::OK, resp_headers, resp_body.clone());

    apply_req(&req, &mut resp);
    assert_eq!(&resp_body, resp.body());
    assert_eq!(None, resp.headers().get("content-encoding"));
    assert_eq!(
        Some(&HeaderValue::NumberUsize(resp_body.len())),
        resp.headers.get("content-length")
    );
}

#[test]
fn apply_already_has_content_encoding() {
    let mut req_headers = Headers::new();
    req_headers.add("Accept-Encoding", "gzip, deflate, br");
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/some/path",
        req_headers,
        "".as_bytes(),
    );
    let resp_body = "test".as_bytes().to_vec();
    let mut resp_headers = Headers::new();
    resp_headers.add("Content-Length", resp_body.len());
    resp_headers.add("Content-Encoding", "gzip");
    let mut resp = Response::new("HTTP/1.1", StatusCode::OK, resp_headers, resp_body.clone());

    apply_req(&req, &mut resp);
    assert_eq!(&resp_body, resp.body());
    assert_eq!(
        Some(&HeaderValue::NumberUsize(resp_body.len())),
        resp.headers.get("content-length")
    );
}

#[test]
fn apply_has_transfer_encoding() {
    let mut req_headers = Headers::new();
    req_headers.add("Accept-Encoding", "gzip, deflate, br");
    let req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/some/path",
        req_headers,
        "".as_bytes(),
    );
    let mut resp_headers = Headers::new();
    resp_headers.add("Transfer-Encoding", "chunked");
    let mut resp = Response::new(
        "HTTP/1.1",
        StatusCode::OK,
        resp_headers,
        "".as_bytes().to_vec(),
    );

    apply_req(&req, &mut resp);
    assert_eq!(None, resp.headers.get("Content-Encoding"));
    assert_eq!(None, resp.headers.get("content-length"));
}
