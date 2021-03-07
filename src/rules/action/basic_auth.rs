use crate::http::{HeaderValue, Headers, Request, Response, StatusCode};

#[cfg(test)]
use crate::http::Method;

fn unauthorized_response(protocol: &str) -> Response {
    let mut headers = Headers::new();
    headers.add("WWW-Authenticate", "Basic realm=\"User Visible Realm\"");

    Response::new(protocol, StatusCode::Unauthorized, headers, vec![])
}

pub fn apply_req<'a>(req: &mut Request<'a>, creds: &str) -> Option<Response<'a>> {
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header,
        None => {
            return Some(unauthorized_response(req.protocol()));
        }
    };

    if !auth_header.eq_ignore_case(&HeaderValue::Str(format!("Basic {}", creds))) {
        return Some(unauthorized_response(req.protocol()));
    }

    None
}

#[test]
fn unauthorized_no_creds() {
    let mut req = Request::new("HTTP/1.1", Method::GET, "/test", Headers::new(), &[]);

    let encoded_creds = "some_test_encoded_stuff";

    let result = apply_req(&mut req, encoded_creds);
    assert_eq!(true, result.is_some());
}

#[test]
fn unauthorized_wrong_creds() {
    let mut headers = Headers::new();
    headers.add("Authorization", "Basic some_other_creds");
    let mut req = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

    let encoded_creds = "some_test_encoded_stuff";

    let result = apply_req(&mut req, encoded_creds);
    assert_eq!(true, result.is_some());
}

#[test]
fn unauthorized_wrong_auth_type() {
    let mut headers = Headers::new();
    headers.add("Authorization", "Bearer some_test_encoded_stuff");
    let mut req = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

    let encoded_creds = "some_test_encoded_stuff";

    let result = apply_req(&mut req, encoded_creds);
    assert_eq!(true, result.is_some());
}

#[test]
fn valid_login() {
    let mut headers = Headers::new();
    headers.add("Authorization", "Basic test_creds");

    let mut req = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

    let encoded_creds = "test_creds";

    let result = apply_req(&mut req, encoded_creds);
    assert_eq!(false, result.is_some());
}
