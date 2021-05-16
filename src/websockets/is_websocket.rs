use stream_httparse::Request;

pub fn is_websocket(req: &Request) -> bool {
    match req.headers().get("Upgrade") {
        Some(val) if val.to_string().to_lowercase() == "websocket" => true,
        _ => false,
    }
}
