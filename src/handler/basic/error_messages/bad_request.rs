use crate::acceptors::traits::Sender;

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
    let (resp_header, resp_body) = response.serialize();
    let resp_header_length = resp_header.len();
    sender.send(resp_header, resp_header_length).await;
    let resp_body_length = resp_body.len();
    sender.send(resp_body.to_vec(), resp_body_length).await;
}
