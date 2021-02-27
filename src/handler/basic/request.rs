use crate::acceptors::traits::Receiver;
use crate::http::streaming_parser::ReqParser;
use crate::http::Request;

use log::error;

/// Returns the finished response and the amount of data
/// still left in the buffer
pub async fn receive<'a, 'b, R>(
    id: u32,
    parser: &'a mut ReqParser,
    rx: &mut R,
    buffer: &mut [u8],
    inital_offset: usize,
) -> Option<(Request<'b>, usize)>
where
    R: Receiver + Send,
    'a: 'b,
{
    let mut continue_parsing = true;
    let mut left_in_buffer: usize = 0;

    if inital_offset > 0 {
        let (done, raw_left_over) = parser.block_parse(&buffer[..inital_offset]);
        if done {
            continue_parsing = false;

            if let Some(left_over) = raw_left_over {
                let start = buffer.len() - left_over;
                buffer.copy_within(start.., 0);
                left_in_buffer = left_over;
            }
        }
    }

    while continue_parsing {
        match rx.read(buffer).await {
            Ok(n) if n == 0 => {
                return None;
            }
            Ok(n) => {
                let (done, raw_left_over) = parser.block_parse(&buffer[..n]);
                if done {
                    continue_parsing = false;

                    if let Some(left_over) = raw_left_over {
                        let start = n - left_over;
                        buffer.copy_within(start..n, 0);
                        left_in_buffer = left_over;
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                error!("[{}] Reading Request: {}", id, e);
                return None;
            }
        };
    }

    match parser.finish() {
        Ok(req) => Some((req, left_in_buffer)),
        Err(e) => {
            error!("[{}] Parsing Request: {}", id, e);
            None
        }
    }
}

#[cfg(test)]
use crate::acceptors::mocks::Receiver as MockReceiver;
#[cfg(test)]
use crate::http::{Headers, Method};

#[tokio::test]
async fn test_valid_one_chunk() {
    let mut tmp_recv = MockReceiver::new();
    tmp_recv.add_chunk(
        "GET /test/ HTTP/1.1\r\nContent-Length: 10\r\nOther-Header: other-value\r\n\r\nJust data."
            .as_bytes()
            .to_vec(),
    );

    // Buffer Large enough to read the entire chunk at once
    let mut read_buf = [0; 2048];
    let initial_offset = 0;
    let id = 0;
    let mut parser = ReqParser::new_capacity(2048);

    // Actually run the function to test
    let result = receive(
        id,
        &mut parser,
        &mut tmp_recv,
        &mut read_buf,
        initial_offset,
    )
    .await;

    assert_eq!(true, result.is_some());
    let (request, left_over) = result.unwrap();

    let mut headers = Headers::new();
    headers.add("Content-Length", 10);
    headers.add("Other-Header", "other-value");
    let expected_req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/test/",
        headers,
        "Just data.".as_bytes(),
    );
    assert_eq!(expected_req, request);

    assert_eq!(0, left_over);
}

#[tokio::test]
async fn test_valid_one_chunk_with_left_over() {
    let mut tmp_recv = MockReceiver::new();
    tmp_recv.add_chunk(
        "GET /test/ HTTP/1.1\r\nContent-Length: 10\r\nOther-Header: other-value\r\n\r\nJust data.And some more"
            .as_bytes()
            .to_vec(),
    );

    // Buffer Large enough to read the entire chunk at once
    let mut read_buf = [0; 2048];
    let initial_offset = 0;
    let id = 0;
    let mut parser = ReqParser::new_capacity(2048);

    // Actually run the function to test
    let result = receive(
        id,
        &mut parser,
        &mut tmp_recv,
        &mut read_buf,
        initial_offset,
    )
    .await;

    assert_eq!(true, result.is_some());
    let (request, left_over) = result.unwrap();

    let mut headers = Headers::new();
    headers.add("Content-Length", 10);
    headers.add("Other-Header", "other-value");
    let expected_req = Request::new(
        "HTTP/1.1",
        Method::GET,
        "/test/",
        headers,
        "Just data.".as_bytes(),
    );
    assert_eq!(expected_req, request);

    assert_eq!(13, left_over);
    assert_eq!("And some more".as_bytes(), &read_buf[..13]);
}
