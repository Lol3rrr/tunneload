use crate::http::Request;

#[cfg(test)]
use crate::http::Method;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    RemovePrefix(String),
    AddHeader(String, String),
}

impl Action {
    pub fn apply(&self, req: &mut Request) {
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
            Self::AddHeader(ref key, ref value) => {
                req.headers.insert(key.clone(), value.clone());
            }
        }
    }
}

#[test]
fn apply_remove_prefix() {
    let mut req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/api/test",
        std::collections::BTreeMap::new(),
        "".as_bytes(),
    );
    let action = Action::RemovePrefix("/api".to_owned());

    action.apply(&mut req);
    assert_eq!("/test", req.path());
}
#[test]
fn apply_remove_prefix_doesnt_exist() {
    let mut req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/test",
        std::collections::BTreeMap::new(),
        "".as_bytes(),
    );
    let action = Action::RemovePrefix("/api".to_owned());

    action.apply(&mut req);
    assert_eq!("/test", req.path());
}
#[test]
fn apply_add_header() {
    let mut headers = std::collections::BTreeMap::new();
    let mut req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/test",
        headers.clone(),
        "".as_bytes(),
    );

    let action = Action::AddHeader("Test-1".to_owned(), "Value-1".to_owned());
    action.apply(&mut req);

    headers.insert("Test-1".to_owned(), "Value-1".to_owned());
    assert_eq!(headers, *req.headers());
}
