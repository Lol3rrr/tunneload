use crate::handler::traits::ServiceConnection;
use crate::http::streaming_parser::RespParser;
use crate::http::Response;

use log::error;

pub async fn receive<'a, 'b, R>(
    id: u32,
    parser: &'a mut RespParser,
    con: &mut R,
) -> Option<(Response<'b>, Option<Vec<u8>>)>
where
    'a: 'b,
    R: ServiceConnection + Send,
{
    let mut left_over_buffer: Option<Vec<u8>> = None;

    let mut read_buffer: [u8; 2048] = [0; 2048];
    loop {
        match con.read(&mut read_buffer).await {
            Ok(n) if n == 0 => {
                return None;
            }
            Ok(n) => {
                let (parser_done, parser_rest) = parser.block_parse(&read_buffer[0..n]);
                if parser_done {
                    if let Some(rest) = parser_rest {
                        left_over_buffer = Some(rest.to_vec());
                    }

                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                error!("[{}] Reading from Connection: {}", id, e);
                return None;
            }
        };
    }

    let result = match parser.finish() {
        Some(r) => r,
        None => return None,
    };
    Some((result, left_over_buffer))
}

#[cfg(test)]
use crate::handler::mocks::ServiceConnection as MockServiceConnection;
#[cfg(test)]
use crate::http::{Headers, StatusCode};

#[tokio::test]
async fn recv_normal_request_no_body() {
    let mut tmp_con = MockServiceConnection::new();
    tmp_con.add_chunk(
        "HTTP/1.1 200 OK\r\nTest-Key: test-value\r\n\r\n"
            .as_bytes()
            .to_vec(),
    );

    let mut parser = RespParser::new_capacity(2048);
    let id = 0;

    let result = receive(id, &mut parser, &mut tmp_con).await;
    assert_eq!(true, result.is_some());

    let (response, left_over_buffer) = result.unwrap();
    let mut headers = Headers::new();
    headers.add("Test-Key", "test-value");
    let expected_response =
        Response::new("HTTP/1.1", StatusCode::OK, headers, "".as_bytes().to_vec());
    assert_eq!(expected_response, response);

    assert_eq!(None, left_over_buffer);
}

#[tokio::test]
async fn recv_normal_request_with_body() {
    let mut tmp_con = MockServiceConnection::new();
    tmp_con.add_chunk(
        "HTTP/1.1 200 OK\r\nTest-Key: test-value\r\nContent-Length: 10\r\n\r\nTest Data."
            .as_bytes()
            .to_vec(),
    );

    let mut parser = RespParser::new_capacity(2048);
    let id = 0;

    let result = receive(id, &mut parser, &mut tmp_con).await;
    assert_eq!(true, result.is_some());

    let (response, left_over_buffer) = result.unwrap();
    let mut headers = Headers::new();
    headers.add("Test-Key", "test-value");
    headers.add("Content-Length", 10);
    let expected_response = Response::new(
        "HTTP/1.1",
        StatusCode::OK,
        headers,
        "Test Data.".as_bytes().to_vec(),
    );
    assert_eq!(expected_response, response);

    assert_eq!(None, left_over_buffer);
}

#[tokio::test]
async fn recv_normal_request_with_body_with_left_over() {
    let mut tmp_con = MockServiceConnection::new();
    tmp_con.add_chunk(
        "HTTP/1.1 200 OK\r\nTest-Key: test-value\r\nContent-Length: 10\r\n\r\nTest Data.Some extra data"
            .as_bytes()
            .to_vec(),
    );

    let mut parser = RespParser::new_capacity(2048);
    let id = 0;

    let result = receive(id, &mut parser, &mut tmp_con).await;
    assert_eq!(true, result.is_some());

    let (response, left_over_buffer) = result.unwrap();
    let mut headers = Headers::new();
    headers.add("Test-Key", "test-value");
    headers.add("Content-Length", 10);
    let expected_response = Response::new(
        "HTTP/1.1",
        StatusCode::OK,
        headers,
        "Test Data.".as_bytes().to_vec(),
    );
    assert_eq!(expected_response, response);

    assert_eq!(
        Some("Some extra data".as_bytes().to_vec()),
        left_over_buffer
    );
}

#[tokio::test]
async fn recv_chunked_request() {
    let mut tmp_con = MockServiceConnection::new();
    tmp_con.add_chunk(
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n10\r\nTest Data.\r\n"
            .as_bytes()
            .to_vec(),
    );

    let mut parser = RespParser::new_capacity(2048);
    let id = 0;

    let result = receive(id, &mut parser, &mut tmp_con).await;
    assert_eq!(true, result.is_some());

    let (response, left_over_buffer) = result.unwrap();
    let mut headers = Headers::new();
    headers.add("Transfer-Encoding", "chunked");
    let expected_response =
        Response::new("HTTP/1.1", StatusCode::OK, headers, "".as_bytes().to_vec());
    assert_eq!(expected_response, response);

    assert_eq!(
        Some("10\r\nTest Data.\r\n".as_bytes().to_vec()),
        left_over_buffer
    );
}
