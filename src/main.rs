use tunneler_core::Destination;

use tunneload::acceptors::tunneler;
use tunneload::handler::BasicHandler;

fn main() {
    env_logger::init();

    let key_file = "./key";

    let raw_key = std::fs::read(key_file).expect("Reading Key File");
    let key = base64::decode(raw_key).unwrap();
    let t_client = tunneler::Client::new(Destination::new("localhost".to_owned(), 8081), key);

    let handler = BasicHandler::new();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(t_client.start(handler));
}
