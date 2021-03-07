use serde_json::Value;

use crate::rules::{Action, Middleware};

pub fn parse(name: &str, value: &Value) -> Option<Middleware> {
    let prefixes = match value.get("prefixes") {
        Some(p) => p.as_array().unwrap(),
        None => {
            log::error!("StripPrefixes is missing 'prefixes'-Entry: {:?}", value);
            return None;
        }
    };
    let raw_prefix = match prefixes.get(0) {
        Some(r) => r,
        None => {
            log::error!("StripPrefixes missing prefix-Entries");
            return None;
        }
    };
    let mut prefix = raw_prefix.as_str().unwrap();

    if prefix.ends_with('/') {
        prefix = &prefix[..prefix.len() - 1];
    }

    Some(Middleware::new(
        name,
        Action::RemovePrefix(prefix.to_owned()),
    ))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn non_trailing_path() {
        let name = "test";
        let value = json!({
            "prefixes": [
                "/test",
            ],
        });

        let result = parse(name, &value);
        assert_eq!(
            Some(Middleware::new(
                "test",
                Action::RemovePrefix("/test".to_owned())
            )),
            result
        );
    }

    #[test]
    fn trailing_path() {
        let name = "test";
        let value = json!({
            "prefixes": [
                "/test/",
            ],
        });

        let result = parse(name, &value);
        assert_eq!(
            Some(Middleware::new(
                "test",
                Action::RemovePrefix("/test".to_owned())
            )),
            result
        );
    }

    #[test]
    fn prefixes_key_missing() {
        let name = "test";
        let value = json!({
            "some_other_key": [
                "other_data",
            ],
        });

        let result = parse(name, &value);
        assert_eq!(None, result);
    }

    #[test]
    fn prefixes_empty() {
        let name = "test";
        let value = json!({
            "prefixes": [],
        });

        let result = parse(name, &value);
        assert_eq!(None, result);
    }
}
