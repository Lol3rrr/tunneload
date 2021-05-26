use crate::{htpasswd, plugins::ActionPluginInstance};

use serde::Serialize;
use stream_httparse::{Request, Response};

mod basic_auth;
mod compress;
mod cors;
mod remove_prefix;

/// The Options to configure CORS
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CorsOpts {
    /// The Origins from which a Request is allowed
    pub origins: Vec<String>,
    /// The Max time a the response of a preflight
    /// Request can be cached
    pub max_age: Option<usize>,
    /// Whether or not to allow Credentials to be send using CORS
    pub credentials: bool,
    /// All the Methods that should be allowed for CORS
    pub methods: Vec<String>,
    /// All the Headers permitted on CORS Requests
    pub headers: Vec<String>,
}

/// An Action performs a specific Mutation on a Request or Response
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", content = "c")]
pub enum Action {
    /// This does nothing and is the default Action
    Noop,
    /// Removes the provided Prefix from the Path of the Request
    RemovePrefix(String),
    /// Adds the List of Headers to every Request or Response
    AddHeaders(Vec<(String, String)>),
    /// Compresses the Response-Body
    Compress,
    /// Allows for the simple use of CORS
    Cors(CorsOpts),
    /// Allows for very basic Authentication of Requests and Users
    BasicAuth(htpasswd::Htpasswd),
    /// This holds an arbitrary Plugin
    Plugin(ActionPluginInstance),
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

    /// Applies the Action to the given Request
    pub fn apply_req<'a, 'b>(&self, req: &mut Request<'a>) -> Result<(), Response<'b>>
    where
        'a: 'b,
    {
        match *self {
            Self::Noop => Ok(()),
            Self::RemovePrefix(ref prefix) => {
                remove_prefix::apply_req(req, prefix);
                Ok(())
            }
            Self::AddHeaders(_) => Ok(()),
            Self::Compress => Ok(()),
            Self::Cors(_) => Ok(()),
            Self::BasicAuth(ref creds) => basic_auth::apply_req(req, creds),
            Self::Plugin(ref instance) => instance.apply_req(req),
        }
    }

    /// Applies the Action to the given Response
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
            Self::Cors(ref opts) => {
                cors::apply_req(req, resp, opts);
            }
            Self::BasicAuth(_) => {}
            Self::Plugin(ref instance) => instance.apply_resp(resp),
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
        assert_eq!(false, action.apply_req(&mut req).is_err());
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
