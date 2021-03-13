use stream_httparse::Request;

pub fn apply_req(req: &mut Request, prefix: &str) {
    let prefix_len = prefix.len();
    let req_path_len = req.path().len();

    if prefix_len > req_path_len {
        return;
    }
    if &req.path()[0..prefix_len] != prefix {
        return;
    }

    req.set_path(&req.path()[prefix_len..]);
}

#[cfg(test)]
mod tests {
    use super::*;

    use stream_httparse::{Headers, Method};

    #[test]
    fn req_valid() {
        let mut req = Request::new(
            "HTTP/1.1",
            Method::GET,
            "/api/test",
            Headers::new(),
            "".as_bytes(),
        );

        apply_req(&mut req, "/api");
        assert_eq!("/test", req.path());
    }
    #[test]
    fn req_doesnt_match_prefix() {
        let mut req = Request::new(
            "HTTP/1.1",
            Method::GET,
            "/test",
            Headers::new(),
            "".as_bytes(),
        );

        apply_req(&mut req, "/api");
        assert_eq!("/test", req.path());
    }
    #[test]
    fn req_path_shorter_than_prefix() {
        let mut req = Request::new("HTTP/1.1", Method::GET, "/", Headers::new(), "".as_bytes());

        apply_req(&mut req, "/api");
        assert_eq!("/", req.path());
    }
}
