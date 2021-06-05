use stream_httparse::Request;

/// Checks if the given Request is the start of a
/// Websocket connection
pub fn is_websocket(req: &Request) -> bool {
    matches!(req.headers().get("Upgrade"), Some(val) if val.to_string().to_lowercase() == "websocket")
}
