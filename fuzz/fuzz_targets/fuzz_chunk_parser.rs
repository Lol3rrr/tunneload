#![no_main]
use libfuzzer_sys::fuzz_target;

use tunneload::http::streaming_parser;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let mut chunk_parser = streaming_parser::ChunkParser::new();

    let (done, _) = chunk_parser.block_parse(data);

    if done {
        chunk_parser.finish();
    }
});
