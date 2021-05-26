use std::path::Path;

use crate::plugins::ActionPlugin;

pub fn load_actions(path: &str) -> Vec<ActionPlugin> {
    let path = Path::new(path);
    let metadata = std::fs::metadata(path).unwrap();
    if metadata.is_file() {
        let raw_file_name = path.file_name().unwrap().to_str().unwrap();
        let (file_name, file_ending) = match raw_file_name.split_once('.') {
            Some(s) => s,
            None => {
                return Vec::new();
            }
        };

        if file_ending != "wasm" {
            return Vec::new();
        }

        let content = match std::fs::read(path) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Reading File: {:?}", e);
                return Vec::new();
            }
        };

        let plugin = match ActionPlugin::new(file_name.to_owned(), &content) {
            Some(p) => p,
            None => {
                log::error!("Loading WASM-Plugin");
                return Vec::new();
            }
        };

        vec![plugin]
    } else {
        let mut result = Vec::new();

        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let raw_path = entry.path();
            let path_str = raw_path.to_str().unwrap();
            result.extend(load_actions(path_str));
        }
        result
    }
}
