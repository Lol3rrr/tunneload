use general_traits::Sender;

use stream_httparse::{Headers, Response, StatusCode};

pub async fn internal_server_error<T>(sender: &mut T)
where
    T: Sender,
{
    let response = Response::new(
        "HTTP/1.1",
        StatusCode::InternalServerError,
        Headers::new(),
        "Internal Server Error".as_bytes().to_vec(),
    );

    sender.send_response(&response).await;
}
