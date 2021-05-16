use stream_httparse::Request;

/// The Websocket related Information parsed out from
/// the initial Handshake Request by the Client
#[derive(Debug, PartialEq)]
pub struct InitialRequest {
    ressource_name: String,
    key: String,
    version: u8,
    origin: Option<String>,
    protocol: Vec<String>,
    extensions: Vec<String>,
}

/// The possible Errors returned while trying to parse
/// the initial Handshake Request by the Client
#[derive(Debug, PartialEq)]
pub enum InitialRequestError {
    /// There was no Host-Header present
    MissingHost,
    /// There was no Upgrade-Header present
    MissingUpgrade,
    /// The Value inside the Upgrade-Header was invalid
    InvalidUpgrade(String),
    /// There was no Connection-Header present
    MissingConnection,
    /// The Value inside the Connection-Header was invalid
    InvalidConnection(String),
    /// There was no Key-Header present
    MissingKey,
    /// The Value inside the Key-Header was invalid
    InvalidKey(String),
    /// There was no Version-Header present
    MissingVersion,
    /// The Value inside the Version-Header was invalid
    InvalidVersion(String),
}

/// Attempts to parse the given Request as the initial
/// Handshake Request by the Client
pub fn parse(req: &Request) -> Result<InitialRequest, InitialRequestError> {
    // Initial validation
    // https://datatracker.ietf.org/doc/html/rfc6455#section-4.2.1

    let headers = req.headers();

    // Host Header
    if let None = headers.get("Host") {
        return Err(InitialRequestError::MissingHost);
    }

    // Upgrade Header
    match headers.get("Upgrade") {
        Some(val) => {
            if val.to_string().to_ascii_lowercase() != "websocket" {
                return Err(InitialRequestError::InvalidUpgrade(val.to_string()));
            }
        }
        None => return Err(InitialRequestError::MissingUpgrade),
    };

    match headers.get("Connection") {
        Some(val) => {
            if val.to_string().to_ascii_lowercase() != "upgrade" {
                return Err(InitialRequestError::InvalidConnection(val.to_string()));
            }
        }
        None => return Err(InitialRequestError::MissingConnection),
    };

    let key = match headers.get("Sec-WebSocket-Key") {
        Some(raw_val) => {
            let raw_str = match raw_val.try_as_str_ref() {
                Some(s) => s,
                None => return Err(InitialRequestError::InvalidKey(raw_val.to_string())),
            };

            let val = match base64::decode(raw_str) {
                Ok(v) => v,
                Err(_) => return Err(InitialRequestError::InvalidKey(raw_val.to_string())),
            };

            if val.len() != 16 {
                return Err(InitialRequestError::InvalidKey(raw_val.to_string()));
            }

            raw_str.to_string()
        }
        None => return Err(InitialRequestError::MissingKey),
    };

    let version = match headers.get("Sec-WebSocket-Version") {
        Some(raw_version) => match raw_version.to_string().parse::<u8>() {
            Ok(v) => v,
            Err(_) => return Err(InitialRequestError::InvalidVersion(raw_version.to_string())),
        },
        None => return Err(InitialRequestError::MissingVersion),
    };

    let origin = headers.get("Origin").map(|o| o.to_string());

    let protocol = match headers.get("Sec-WebSocket-Protocol") {
        Some(raw_protocols) => {
            let protocols = raw_protocols.to_string();

            let mut result = Vec::new();
            for tmp in protocols.split(',') {
                let tmp = tmp.trim_start().trim_end();
                result.push(tmp.to_string());
            }

            result
        }
        None => Vec::new(),
    };

    let extension = match headers.get("Sec-WebSocket-Extensions") {
        Some(raw_extensions) => {
            let extensions = raw_extensions.to_string();

            let mut result = Vec::new();
            for tmp in extensions.split(',') {
                let tmp = tmp.trim_start().trim_end();
                result.push(tmp.to_string());
            }

            result
        }
        None => Vec::new(),
    };

    Ok(InitialRequest {
        ressource_name: req.path().to_string(),
        key,
        version,
        origin,
        protocol,
        extensions: extension,
    })
}

#[cfg(test)]
mod tests {
    use stream_httparse::{Headers, Method};

    use super::*;

    #[test]
    fn no_host_header() {
        let headers = Headers::new();
        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(Err(InitialRequestError::MissingHost), parse(&request));
    }
    #[test]
    fn no_upgrade_header() {
        let mut headers = Headers::new();
        headers.append("Host", "test authority");
        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(Err(InitialRequestError::MissingUpgrade), parse(&request));
    }
    #[test]
    fn invalid_upgrade_header() {
        let mut headers = Headers::new();
        headers.append("Host", "test authority");
        headers.append("Upgrade", "other");
        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(
            Err(InitialRequestError::InvalidUpgrade("other".to_string())),
            parse(&request)
        );
    }
    #[test]
    fn no_connection_header() {
        let mut headers = Headers::new();
        headers.append("Host", "test authority");
        headers.append("Upgrade", "websocket");
        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(Err(InitialRequestError::MissingConnection), parse(&request));
    }
    #[test]
    fn invalid_connection_header() {
        let mut headers = Headers::new();
        headers.append("Host", "test authority");
        headers.append("Upgrade", "websocket");
        headers.append("Connection", "other");
        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(
            Err(InitialRequestError::InvalidConnection("other".to_string())),
            parse(&request)
        );
    }
    #[test]
    fn no_key() {
        let mut headers = Headers::new();
        headers.append("Host", "test authority");
        headers.append("Upgrade", "websocket");
        headers.append("Connection", "Upgrade");

        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(Err(InitialRequestError::MissingKey), parse(&request));
    }
    #[test]
    fn invalid_key() {
        let mut headers = Headers::new();
        headers.append("Host", "test authority");
        headers.append("Upgrade", "websocket");
        headers.append("Connection", "Upgrade");
        headers.append("Sec-WebSocket-Key", "aW52YWxpZA=="); // "invalid" in base64

        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(
            Err(InitialRequestError::InvalidKey("aW52YWxpZA==".to_string())),
            parse(&request)
        );
    }
    #[test]
    fn no_version() {
        let mut headers = Headers::new();
        headers.append("Host", "test authority");
        headers.append("Upgrade", "websocket");
        headers.append("Connection", "Upgrade");
        headers.append("Sec-WebSocket-Key", "d2lydHNjaGFmdHNrdW5kZQ=="); // "wirtschaftskunde" in base64

        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(Err(InitialRequestError::MissingVersion), parse(&request));
    }
    #[test]
    fn malformed_version_string() {
        let mut headers = Headers::new();
        headers.append("Host", "test authority");
        headers.append("Upgrade", "websocket");
        headers.append("Connection", "Upgrade");
        headers.append("Sec-WebSocket-Key", "d2lydHNjaGFmdHNrdW5kZQ=="); // "wirtschaftskunde" in base64
        headers.append("Sec-WebSocket-Version", "test");

        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(
            Err(InitialRequestError::InvalidVersion("test".to_string())),
            parse(&request)
        );
    }

    #[test]
    fn minimal_valid_request() {
        let mut headers = Headers::new();
        headers.append("Host", "test authority");
        headers.append("Upgrade", "websocket");
        headers.append("Connection", "Upgrade");
        headers.append("Sec-WebSocket-Key", "d2lydHNjaGFmdHNrdW5kZQ=="); // "wirtschaftskunde" in base64
        headers.append("Sec-WebSocket-Version", "13");

        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(
            Ok(InitialRequest {
                ressource_name: "/test".to_string(),
                key: "d2lydHNjaGFmdHNrdW5kZQ==".to_string(),
                version: 13,
                origin: None,
                protocol: vec![],
                extensions: vec![],
            }),
            parse(&request)
        );
    }

    #[test]
    fn full_valid_request() {
        let mut headers = Headers::new();
        headers.append("Host", "test authority");
        headers.append("Upgrade", "websocket");
        headers.append("Connection", "Upgrade");
        headers.append("Sec-WebSocket-Key", "d2lydHNjaGFmdHNrdW5kZQ=="); // "wirtschaftskunde" in base64
        headers.append("Sec-WebSocket-Version", "13");
        headers.append("Origin", "test origin");
        headers.append("Sec-WebSocket-Protocol", "chat, superchat");
        headers.append(
            "Sec-WebSocket-Extensions",
            "test-extension1, test-extension2",
        );

        let request = Request::new("HTTP/1.1", Method::GET, "/test", headers, &[]);

        assert_eq!(
            Ok(InitialRequest {
                ressource_name: "/test".to_string(),
                key: "d2lydHNjaGFmdHNrdW5kZQ==".to_string(),
                version: 13,
                origin: Some("test origin".to_string()),
                protocol: vec!["chat".to_string(), "superchat".to_string()],
                extensions: vec!["test-extension1".to_string(), "test-extension2".to_string()],
            }),
            parse(&request)
        );
    }
}
