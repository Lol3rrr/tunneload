use crate::rules::action::CorsOpts;

use stream_httparse::{Request, Response};

fn valid_origin(origin: &str, valid: &[String]) -> bool {
    for tmp in valid.iter() {
        if tmp == origin {
            return true;
        }
    }

    false
}

/// Actually applies the CORS settings accordingly
pub fn apply_req(req: &Request<'_>, resp: &mut Response<'_>, opts: &CorsOpts) {
    let origin = match req.headers().get("Origin") {
        Some(o) => o,
        None => {
            return;
        }
    };

    let origin_value = origin.to_string();
    if !valid_origin(&origin_value, &opts.origins) {
        return;
    }
    resp.add_header("Access-Control-Allow-Origin", origin_value);

    if let Some(max_age) = opts.max_age {
        resp.add_header("Access-Control-Max-Age", max_age);
    }

    if opts.credentials {
        resp.add_header("Access-Control-Allow-Credentials", "true");
    }

    if !opts.methods.is_empty() {
        let mut methods_value = String::new();
        for tmp_method in &opts.methods {
            methods_value.push_str(tmp_method.as_str());
            methods_value.push_str(", ");
        }
        methods_value.pop();
        methods_value.pop();

        resp.add_header("Access-Control-Allow-Methods", methods_value);
    }

    if !opts.headers.is_empty() {
        let mut headers_value = String::new();
        for tmp_header in &opts.headers {
            headers_value.push_str(tmp_header.as_str());
            headers_value.push_str(", ");
        }
        headers_value.pop();
        headers_value.pop();

        resp.add_header("Access-Control-Allow-Headers", headers_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use stream_httparse::{header::HeaderValue, Headers, Method, StatusCode};

    #[test]
    fn apply_req_valid() {
        let mut req_headers = Headers::new();
        req_headers.set("Origin", "http://localhost");
        let request = Request::new(
            "HTTP/1.1",
            Method::GET,
            "/path/",
            req_headers,
            "".as_bytes(),
        );

        let mut response = Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            Headers::new(),
            "".as_bytes().to_vec(),
        );

        let cors_opts = CorsOpts {
            origins: vec![
                "http://localhost".to_owned(),
                "http://example.com".to_owned(),
            ],
            max_age: Some(3600),
            credentials: true,
            methods: vec!["GET".to_owned()],
            headers: vec!["X-Requested-With".to_owned()],
        };

        apply_req(&request, &mut response, &cors_opts);

        let origin_header = response.headers().get("Access-Control-Allow-Origin");
        assert_eq!(true, origin_header.is_some());
        assert_eq!(
            true,
            HeaderValue::StrRef("http://localhost").eq_ignore_case(&origin_header.unwrap())
        );

        let max_age_header = response.headers().get("Access-Control-Max-Age");
        assert_eq!(true, max_age_header.is_some());
        assert_eq!(&HeaderValue::NumberUsize(3600), max_age_header.unwrap());

        let credentials_header = response.headers().get("Access-Control-Allow-Credentials");
        assert_eq!(true, credentials_header.is_some());
        assert_eq!(
            true,
            HeaderValue::StrRef("true").eq_ignore_case(&credentials_header.unwrap())
        );

        let methods_header = response.headers().get("Access-Control-Allow-Methods");
        assert_eq!(true, methods_header.is_some());
        assert_eq!(
            true,
            HeaderValue::StrRef("GET").eq_ignore_case(&methods_header.unwrap())
        );

        let headers_header = response.headers().get("Access-Control-Allow-Headers");
        assert_eq!(true, headers_header.is_some());
        assert_eq!(
            true,
            HeaderValue::StrRef("X-Requested-With").eq_ignore_case(&headers_header.unwrap())
        );
    }

    #[test]
    fn apply_req_no_origin_set() {
        let request = Request::new(
            "HTTP/1.1",
            Method::GET,
            "/path/",
            Headers::new(),
            "".as_bytes(),
        );

        let mut response = Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            Headers::new(),
            "".as_bytes().to_vec(),
        );

        let cors_opts = CorsOpts {
            origins: vec![
                "http://localhost".to_owned(),
                "http://example.com".to_owned(),
            ],
            max_age: Some(3600),
            credentials: true,
            methods: vec!["GET".to_owned()],
            headers: vec!["X-Requested-With".to_owned()],
        };

        apply_req(&request, &mut response, &cors_opts);

        let origin_header = response.headers().get("Access-Control-Allow-Origin");
        assert_eq!(false, origin_header.is_some());

        let max_age_header = response.headers().get("Access-Control-Max-Age");
        assert_eq!(false, max_age_header.is_some());

        let credentials_header = response.headers().get("Access-Control-Allow-Credentials");
        assert_eq!(false, credentials_header.is_some());

        let methods_header = response.headers().get("Access-Control-Allow-Methods");
        assert_eq!(false, methods_header.is_some());

        let headers_header = response.headers().get("Access-Control-Allow-Headers");
        assert_eq!(false, headers_header.is_some());
    }
    #[test]
    fn apply_req_invalid_origin_set() {
        let mut req_headers = Headers::new();
        req_headers.set("Origin", "http://other.net");
        let request = Request::new(
            "HTTP/1.1",
            Method::GET,
            "/path/",
            req_headers,
            "".as_bytes(),
        );

        let mut response = Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            Headers::new(),
            "".as_bytes().to_vec(),
        );

        let cors_opts = CorsOpts {
            origins: vec![
                "http://localhost".to_owned(),
                "http://example.com".to_owned(),
            ],
            max_age: Some(3600),
            credentials: true,
            methods: vec!["GET".to_owned()],
            headers: vec!["X-Requested-With".to_owned()],
        };

        apply_req(&request, &mut response, &cors_opts);

        let origin_header = response.headers().get("Access-Control-Allow-Origin");
        assert_eq!(false, origin_header.is_some());

        let max_age_header = response.headers().get("Access-Control-Max-Age");
        assert_eq!(false, max_age_header.is_some());

        let credentials_header = response.headers().get("Access-Control-Allow-Credentials");
        assert_eq!(false, credentials_header.is_some());

        let methods_header = response.headers().get("Access-Control-Allow-Methods");
        assert_eq!(false, methods_header.is_some());

        let headers_header = response.headers().get("Access-Control-Allow-Headers");
        assert_eq!(false, headers_header.is_some());
    }
}
