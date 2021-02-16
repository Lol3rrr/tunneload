use crate::http::{Headers, Request, Response};

#[cfg(test)]
use crate::http::{Method, StatusCode};

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    RemovePrefix(String),
    AddHeader(String, String),
}

impl Action {
    pub fn apply_req(&self, req: &mut Request) {
        match *self {
            Self::RemovePrefix(ref prefix) => {
                let prefix_len = prefix.len();
                let req_path_len = req.path().len();

                if prefix_len > req_path_len {
                    return;
                }
                if &req.path()[0..prefix_len] != prefix.as_str() {
                    return;
                }

                req.path = &req.path[prefix_len..];
            }
            Self::AddHeader(_, _) => {}
        }
    }

    pub fn apply_resp<'a, 'b, 'c>(&'a self, resp: &'b mut Response<'c>)
    where
        'a: 'b,
        'a: 'c,
        'c: 'b,
    {
        match *self {
            Self::RemovePrefix(_) => {}
            Self::AddHeader(ref key, ref value) => {
                resp.add_header(key, value);
            }
        }
    }
}

#[test]
fn apply_req_remove_prefix() {
    let mut req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/api/test",
        Headers::new(),
        "".as_bytes(),
    );
    let action = Action::RemovePrefix("/api".to_owned());

    action.apply_req(&mut req);
    assert_eq!("/test", req.path());
}
#[test]
fn apply_req_remove_prefix_doesnt_exist() {
    let mut req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/test",
        Headers::new(),
        "".as_bytes(),
    );
    let action = Action::RemovePrefix("/api".to_owned());

    action.apply_req(&mut req);
    assert_eq!("/test", req.path());
}

#[test]
fn apply_req_add_header() {
    let headers = Headers::new();
    let mut req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/test",
        headers.clone(),
        "".as_bytes(),
    );

    // This is expected to do nothing, as the AddHeader Action only performs
    // actions on Responses not Requests
    let action = Action::AddHeader("Test-1".to_owned(), "Value-1".to_owned());
    action.apply_req(&mut req);
    assert_eq!(headers, *req.headers());
}
#[test]
fn apply_resp_add_header() {
    let mut headers = Headers::new();
    let mut resp = Response::new("HTTP/1.1", StatusCode::OK, headers.clone(), "".as_bytes());

    let action = Action::AddHeader("Test-1".to_owned(), "Value-1".to_owned());
    action.apply_resp(&mut resp);

    headers.add("Test-1", "Value-1");
    assert_eq!(headers, resp.headers);
}
