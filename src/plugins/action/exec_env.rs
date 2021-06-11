#[derive(Debug)]
pub enum MiddlewareOp {
    SetPath(String),
    SetHeader(String, String),
    SetBody(Vec<u8>),
}
