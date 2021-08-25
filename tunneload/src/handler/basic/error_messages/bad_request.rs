use general_traits::Sender;

use stream_httparse::{Headers, Response, StatusCode};

pub async fn bad_request<T>(sender: &mut T)
where
    T: Sender,
{
    let response = Response::new(
        "HTTP/1.1",
        StatusCode::BadRequest,
        Headers::new(),
        "Bad Request".as_bytes().to_vec(),
    );

    sender.send_response(&response).await;
}
