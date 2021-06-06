use crate::configurator::{files::Config, parser::RawMiddlewareConfig};

pub fn parse_middleware(tmp_middle: &serde_json::Value) -> Option<Vec<RawMiddlewareConfig>> {
    let name = tmp_middle.get("name")?;
    let name = name.as_str()?;

    let tmp_obj = tmp_middle.as_object()?;

    let cap = tmp_obj.keys().len().saturating_sub(1);
    let mut result = Vec::with_capacity(cap);
    for key in tmp_obj.keys() {
        if key == "name" {
            continue;
        }

        let value = tmp_obj.get(key).unwrap();

        result.push(RawMiddlewareConfig {
            name: name.to_string(),
            action_name: key.to_string(),
            config: value.to_owned(),
        });
    }

    Some(result)
}

pub fn load_file(path: &str) -> Option<Vec<RawMiddlewareConfig>> {
    let content = match std::fs::read(path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Loading File: {:?}", e);
            return None;
        }
    };

    let deserialized: Config = match serde_yaml::from_slice(&content) {
        Ok(d) => d,
        Err(e) => {
            log::error!("Deserialize Config: {:?}", e);
            return None;
        }
    };

    if deserialized.middleware.is_none() {
        return Some(Vec::new());
    }

    let middlewares = deserialized.middleware?;
    let mut result = Vec::new();
    for tmp in middlewares.iter() {
        match parse_middleware(tmp) {
            Some(midds) => result.extend(midds),
            None => {
                log::error!("Parsing Middlewares");
                continue;
            }
        };
    }

    Some(result)
}
