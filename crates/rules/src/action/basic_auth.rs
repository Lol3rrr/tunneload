use stream_httparse::{Headers, Request, Response, StatusCode};

fn forbidden_response(protocol: &str) -> Response {
    let mut headers = Headers::new();
    headers.set("Content-Length", 0);

    Response::new(protocol, StatusCode::Forbidden, headers, vec![])
}

fn unauthorized_response(protocol: &str) -> Response {
    let mut headers = Headers::new();
    headers.set("WWW-Authenticate", "Basic realm=\"tunneload\"");
    headers.set("Content-Length", 0);

    Response::new(protocol, StatusCode::Unauthorized, headers, vec![])
}

/// The Creds are in the Format
/// (username, hashed_password)
///
/// The Password should be hashed using SHA-1
pub fn apply_req<'a>(
    req: &mut Request<'a>,
    creds: &htpasswd::Htpasswd,
) -> Result<(), Response<'a>> {
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header,
        None => return Err(unauthorized_response(req.protocol())),
    };

    let auth_str = match auth_header.try_as_str_ref() {
        Some(a) => a,
        None => return Err(unauthorized_response(req.protocol())),
    };

    let end_of_basic = match auth_str.find(' ') {
        Some(i) => i,
        None => return Err(forbidden_response(req.protocol())),
    };

    let (auth_type, raw_auth_content) = auth_str.split_at(end_of_basic + 1);
    if !auth_type.eq_ignore_ascii_case("Basic ") {
        return Err(forbidden_response(req.protocol()));
    }

    let decoded_auth_content = match base64::decode_config(raw_auth_content, base64::URL_SAFE) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Decoding Authorization Header: {}", e);
            return Err(forbidden_response(req.protocol()));
        }
    };

    let auth_content = match std::str::from_utf8(&decoded_auth_content) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Decoding Authorization Header: {}", e);
            return Err(forbidden_response(req.protocol()));
        }
    };

    let creds_split_point = match auth_content.find(':') {
        Some(i) => i,
        None => {
            tracing::error!("Invalid Credentials-Format");
            return Err(forbidden_response(req.protocol()));
        }
    };

    let (username, raw_password) = auth_content.split_at(creds_split_point);
    let password = &raw_password[1..];

    if !creds.check(username, password) {
        tracing::error!("Invalid Credentials");
        return Err(forbidden_response(req.protocol()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use stream_httparse::Method;

    #[test]
    fn unauthorized_no_creds() {
        let mut req = Request::new("HTTP/1.1", Method::GET, "/test", Headers::new(), &[]);

        let cred_str = format!(
            "{}:{}",
            "user",
            htpasswd::md5::format_hash(
                &htpasswd::md5::md5_apr1_encode("password", "testSalt").unwrap(),
                "testSalt"
            )
        );

        let result = apply_req(&mut req, &htpasswd::load(&cred_str));
        assert_eq!(true, result.is_err());
    }

    #[test]
    fn unauthorized_creds_not_base64() {
        let mut headers = Headers::new();
        headers.set("Authorization", "Basic user:password");
        let mut req = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        let cred_str = format!(
            "{}:{}",
            "user",
            htpasswd::md5::format_hash(
                &htpasswd::md5::md5_apr1_encode("password", "testSalt").unwrap(),
                "testSalt"
            )
        );

        let result = apply_req(&mut req, &htpasswd::load(&cred_str));
        assert_eq!(true, result.is_err());
    }

    #[test]
    fn unauthorized_wrong_creds() {
        let mut headers = Headers::new();
        headers.set(
            "Authorization",
            format!("Basic {}", base64::encode(b"other_user:just_some_data")),
        );
        let mut req = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        let cred_str = format!(
            "{}:{}",
            "user",
            htpasswd::md5::format_hash(
                &htpasswd::md5::md5_apr1_encode("password", "testSalt").unwrap(),
                "testSalt"
            )
        );

        let result = apply_req(&mut req, &htpasswd::load(&cred_str));
        assert_eq!(true, result.is_err());
    }

    #[test]
    fn unauthorized_wrong_auth_type() {
        let mut headers = Headers::new();
        headers.set("Authorization", "Bearer some_test_encoded_stuff");
        let mut req = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        let cred_str = format!(
            "{}:{}",
            "user",
            htpasswd::md5::format_hash(
                &htpasswd::md5::md5_apr1_encode("password", "testSalt").unwrap(),
                "testSalt"
            )
        );

        let result = apply_req(&mut req, &htpasswd::load(&cred_str));
        assert_eq!(true, result.is_err());
    }

    #[test]
    fn valid_login() {
        let mut headers = Headers::new();
        headers.set(
            "Authorization",
            format!("Basic {}", base64::encode(b"user:password")),
        );

        let mut req = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        let cred_str = format!(
            "{}:{}",
            "user",
            htpasswd::md5::format_hash(
                &htpasswd::md5::md5_apr1_encode("password", "testSalt").unwrap(),
                "testSalt"
            )
        );

        let result = apply_req(&mut req, &htpasswd::load(&cred_str));
        assert_eq!(false, result.is_err());
    }
}
