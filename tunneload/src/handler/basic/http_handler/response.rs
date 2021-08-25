use crate::forwarder::ServiceConnection;

use stream_httparse::{streaming_parser::RespParser, Response};

/// Returns the finished response and the amount of
/// data still left in the Buffer
#[tracing::instrument(skip(parser, con, read_buf))]
pub async fn receive<'a, 'b, R>(
    id: u32,
    parser: &'a mut RespParser,
    con: &mut R,
    read_buf: &mut [u8],
) -> Option<(Response<'b>, usize)>
where
    'a: 'b,
    R: ServiceConnection + Send,
{
    let mut left_in_buffer: usize = 0;

    loop {
        match con.read(read_buf).await {
            Ok(n) if n == 0 => {
                return None;
            }
            Ok(n) => {
                let (done, left_over) = parser.block_parse(&read_buf[0..n]);
                if done {
                    if left_over > 0 {
                        let start = n - left_over;
                        read_buf.copy_within(start.., 0);
                        left_in_buffer = left_over;
                    }

                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                tracing::error!("Reading from Connection: {}", e);
                return None;
            }
        };
    }

    let result = match parser.finish() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Finishing Parser: {}", e);
            return None;
        }
    };
    Some((result, left_in_buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::forwarder::mocks::ServiceConnection as MockServiceConnection;

    use stream_httparse::{Headers, StatusCode};

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
        let mut buf = [0; 2048];

        let result = receive(id, &mut parser, &mut tmp_con, &mut buf).await;
        assert_eq!(true, result.is_some());

        let (response, left_over_buffer) = result.unwrap();
        let mut headers = Headers::new();
        headers.set("Test-Key", "test-value");
        let expected_response =
            Response::new("HTTP/1.1", StatusCode::OK, headers, "".as_bytes().to_vec());
        assert_eq!(expected_response, response);

        assert_eq!(0, left_over_buffer);
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
        let mut buf = [0; 2048];

        let result = receive(id, &mut parser, &mut tmp_con, &mut buf).await;
        assert_eq!(true, result.is_some());

        let (response, left_over_buffer) = result.unwrap();
        let mut headers = Headers::new();
        headers.set("Test-Key", "test-value");
        headers.set("Content-Length", 10);
        let expected_response = Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers,
            "Test Data.".as_bytes().to_vec(),
        );
        assert_eq!(expected_response, response);

        assert_eq!(0, left_over_buffer);
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
        let mut buf = [0; 2048];

        let result = receive(id, &mut parser, &mut tmp_con, &mut buf).await;
        assert_eq!(true, result.is_some());

        let (response, left_over_buffer) = result.unwrap();
        let mut headers = Headers::new();
        headers.set("Test-Key", "test-value");
        headers.set("Content-Length", 10);
        let expected_response = Response::new(
            "HTTP/1.1",
            StatusCode::OK,
            headers,
            "Test Data.".as_bytes().to_vec(),
        );
        assert_eq!(expected_response, response);

        assert_eq!(15, left_over_buffer);
        assert_eq!("Some extra data".as_bytes(), &buf[0..15]);
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
        let mut buf = [0; 2048];

        let result = receive(id, &mut parser, &mut tmp_con, &mut buf).await;
        assert_eq!(true, result.is_some());

        let (response, left_over_buffer) = result.unwrap();
        let mut headers = Headers::new();
        headers.set("Transfer-Encoding", "chunked");
        let expected_response =
            Response::new("HTTP/1.1", StatusCode::OK, headers, "".as_bytes().to_vec());
        assert_eq!(expected_response, response);

        assert_eq!(16, left_over_buffer);
        assert_eq!("10\r\nTest Data.\r\n".as_bytes(), &buf[0..16]);
    }
}
