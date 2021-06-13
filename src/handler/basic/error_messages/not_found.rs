use crate::acceptors::traits::Sender;

use stream_httparse::{Headers, Response, StatusCode};

pub async fn not_found<T>(sender: &mut T)
where
    T: Sender,
{
    let response = Response::new(
        "HTTP/1.1",
        StatusCode::NotFound,
        Headers::new(),
        "Not Found".as_bytes().to_vec(),
    );

    sender.send_response(&response).await;
}
