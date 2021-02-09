use crate::http::Request;

#[cfg(test)]
use crate::http::{Header, Method};

#[derive(Clone, Debug, PartialEq)]
pub enum Middleware {
    RemovePrefix(String),
}

impl Middleware {
    pub fn apply(&self, req: &mut Request) {
        match *self {
            Middleware::RemovePrefix(ref prefix) => {
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
        }
    }
}

#[test]
fn apply_remove_prefix() {
    let mut req = Request::new("HTTP/1.1", Method::GET, "/api/test", vec![], "".as_bytes());
    let middleware = Middleware::RemovePrefix("/api".to_owned());

    middleware.apply(&mut req);
    assert_eq!("/test", req.path());
}
#[test]
fn apply_remove_prefix_doesnt_exist() {
    let mut req = Request::new("HTTP/1.1", Method::GET, "/test", vec![], "".as_bytes());
    let middleware = Middleware::RemovePrefix("/api".to_owned());

    middleware.apply(&mut req);
    assert_eq!("/test", req.path());
}
