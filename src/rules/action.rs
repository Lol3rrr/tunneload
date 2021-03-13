use crate::htpasswd;

use stream_httparse::{Request, Response};

mod basic_auth;
mod compress;
mod cors;
mod remove_prefix;

#[derive(Clone, Debug, PartialEq)]
pub struct CorsOpts {
    pub origins: Vec<String>,
    pub max_age: Option<usize>,
    pub credentials: bool,
    pub methods: Vec<String>,
    pub headers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Noop,
    RemovePrefix(String),
    AddHeaders(Vec<(String, String)>),
    Compress,
    CORS(CorsOpts),
    BasicAuth(htpasswd::Htpasswd),
}

impl Action {
    /// Creates a Basic-Auth Action with an already Hashed
    /// password
    pub fn new_basic_auth_hashed<S>(htpasswd_str: S) -> Action
    where
        S: AsRef<str>,
    {
        Action::BasicAuth(htpasswd::load(htpasswd_str.as_ref()))
    }

    pub fn apply_req<'a>(&self, req: &mut Request<'a>) -> Option<Response<'a>> {
        match *self {
            Self::Noop => None,
            Self::RemovePrefix(ref prefix) => {
                remove_prefix::apply_req(req, prefix);
                None
            }
            Self::AddHeaders(_) => None,
            Self::Compress => None,
            Self::CORS(_) => None,
            Self::BasicAuth(ref creds) => basic_auth::apply_req(req, creds),
        }
    }

    pub fn apply_resp<'a, 'b, 'c>(&'a self, req: &Request<'_>, resp: &'b mut Response<'c>)
    where
        'a: 'b,
        'a: 'c,
        'c: 'b,
    {
        match *self {
            Self::Noop => {}
            Self::RemovePrefix(_) => {}
            Self::AddHeaders(ref headers) => {
                for (key, value) in headers {
                    resp.add_header(key.as_str(), value.as_str());
                }
            }
            Self::Compress => {
                compress::apply_req(req, resp);
            }
            Self::CORS(ref opts) => {
                cors::apply_req(req, resp, opts);
            }
            Self::BasicAuth(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use stream_httparse::{Headers, Method, StatusCode};

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
        let action = Action::AddHeaders(vec![("Test-1".to_owned(), "Value-1".to_owned())]);
        assert_eq!(false, action.apply_req(&mut req).is_some());
        assert_eq!(headers, *req.headers());
    }
    #[test]
    fn apply_resp_add_header() {
        let req = Request::new(
            "HTTP/1.1",
            Method::GET,
            "/test",
            Headers::new(),
            "".as_bytes(),
        );

        let mut headers = Headers::new();
        let mut resp = Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers.clone(),
            "".as_bytes().to_vec(),
        );

        let action = Action::AddHeaders(vec![("Test-1".to_owned(), "Value-1".to_owned())]);
        action.apply_resp(&req, &mut resp);

        headers.set("Test-1", "Value-1");
        assert_eq!(&headers, resp.headers());
    }
}
