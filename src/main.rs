use tunneler_core::Destination;

use tunneload::acceptors::tunneler;
use tunneload::configurator;
use tunneload::general;
use tunneload::handler::BasicHandler;
use tunneload::rules;

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

    let server_addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "localhost".to_owned());
    let raw_server_port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8081".to_owned());
    let server_port: u32 = raw_server_port.parse().unwrap();

    let raw_key = std::fs::read(key_file).expect("Reading Key File");
    let key = base64::decode(raw_key).unwrap();
    let t_client = tunneler::Client::new(Destination::new(server_addr, server_port), key);

    let (read_manager, write_manager) = rules::new();

    let handler = BasicHandler::new(read_manager);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let k8s_manager = rt.block_on(configurator::kubernetes::Loader::new("default".to_owned()));
    let config_manager = configurator::Manager::new(vec![Box::new(k8s_manager)], write_manager);
    let config_wait_time =
        general::parse_time(&std::env::var("K8S_UTIME").unwrap_or_else(|_| "30s".to_owned()))
            .unwrap();
    rt.spawn(config_manager.update_loop(config_wait_time));

    rt.block_on(t_client.start(handler));
}
