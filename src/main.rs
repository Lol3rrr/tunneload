use tunneler_core::Destination;

use tunneload::acceptors::tunneler;
use tunneload::handler::BasicHandler;
use tunneload::rules::{Manager, Matcher, Rule, Service};

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

    let test_rule = Rule::new(
        Matcher::Domain("localhost:8080".to_owned()),
        Service::new("localhost:8090".to_owned()),
    );

    let rules_manager = std::sync::Arc::new(Manager::new());
    rules_manager.add_rule(test_rule);

    let handler = BasicHandler::new(rules_manager);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(t_client.start(handler));
}
