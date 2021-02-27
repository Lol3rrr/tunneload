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
