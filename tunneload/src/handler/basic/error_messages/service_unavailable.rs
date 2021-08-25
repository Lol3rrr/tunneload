use general_traits::Sender;

use stream_httparse::{Headers, Response, StatusCode};

pub async fn service_unavailable<T>(sender: &mut T)
where
    T: Sender,
{
    let response = Response::new(
        "HTTP/1.1",
        StatusCode::ServiceUnavailable,
        Headers::new(),
        "Service Unavailable".as_bytes().to_vec(),
    );

    sender.send_response(&response).await;
}
