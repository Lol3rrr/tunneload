use crate::configurator::{files::Config, parser::RawRuleConfig};

pub fn load_file(path: &str) -> Option<Vec<RawRuleConfig>> {
    let content = match std::fs::read(path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Reading File: {:?}", e);
            return None;
        }
    };

    let value: Config = match serde_yaml::from_slice(&content) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Parsing YAML: {:?}", e);
            return None;
        }
    };

    if value.routes.is_none() {
        return None;
    }

    let mut result = Vec::new();
    for tmp in value.routes.unwrap() {
        let tmp_value = serde_json::to_value(tmp).unwrap();
        result.push(RawRuleConfig { config: tmp_value });
    }

    Some(result)
}
