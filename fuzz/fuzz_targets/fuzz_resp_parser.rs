#![no_main]
use libfuzzer_sys::fuzz_target;

use tunneload::http::streaming_parser;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let mut resp_parser = streaming_parser::RespParser::new_capacity(2048);

    let (done, _) = resp_parser.block_parse(data);

    if done {
        resp_parser.finish();
    }
});
