use crate::configurator::{files::Config, parser::RawRuleConfig};

pub fn load_file(content: Vec<u8>) -> Option<Vec<RawRuleConfig>> {
    let value: Config = match serde_yaml::from_slice(&content) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Parsing YAML: {:?}", e);
            return None;
        }
    };

    let routes = value.routes?;

    let mut result = Vec::new();
    for tmp in routes {
        let tmp_value = match serde_json::to_value(tmp) {
            Ok(v) => v,
            Err(_) => continue,
        };
        result.push(RawRuleConfig { config: tmp_value });
    }

    Some(result)
}
