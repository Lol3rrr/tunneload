#![no_main]
use libfuzzer_sys::fuzz_target;

use tunneload::http::streaming_parser;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let mut req_parser = streaming_parser::ReqParser::new_capacity(2048);

    let (done, _) = req_parser.block_parse(data);

    if done {
        req_parser.finish();
    }
});
