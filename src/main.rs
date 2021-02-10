use tunneler_core::Destination;

use tunneload::acceptors::tunneler;
use tunneload::handler::BasicHandler;
use tunneload::kubernetes;
use tunneload::rules::{self};

fn main() {
    env_logger::init();

    let key_file = match std::env::var("KEY_FILE") {
        Ok(val) => val,
        Err(_) => {
            let mut key_path = dirs::home_dir().unwrap();
            key_path.push(".tunneler");
            key_path.push("key");
            key_path.as_path().to_str().unwrap().to_string()
        }
    };

    let raw_key = std::fs::read(key_file).expect("Reading Key File");
    let key = base64::decode(raw_key).unwrap();
    let t_client = tunneler::Client::new(Destination::new("localhost".to_owned(), 8081), key);

    let (read_manager, write_manager) = rules::new();

    let handler = BasicHandler::new(read_manager);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let k8s_manager = rt.block_on(kubernetes::Manager::new());
    rt.spawn(k8s_manager.update_loop(write_manager));

    rt.block_on(t_client.start(handler));
}
