use crate::rules::Matcher;

pub fn parse_matcher_rule(raw: &str) -> Option<Vec<Matcher>> {
    let mut result = Vec::new();

    let parts = raw.split("&&");
    for part in parts {
        let key_end = part.find('(').unwrap_or_else(|| part.len());
        let raw_key = part[0..key_end].to_owned();
        let key = raw_key.replace(' ', "");

        let content_parts: Vec<&str> = part.split('`').collect();
        let content = content_parts.get(1).unwrap();

        match key.as_str() {
            "Host" => {
                result.push(Matcher::Domain(content.to_string()));
            }
            "PathPrefix" => {
                result.push(Matcher::PathPrefix(content.to_string()));
            }
            _ => {
                println!("Unknown: '{}'", part);
            }
        };
    }

    Some(result)
}

#[test]
fn parse_single() {
    assert_eq!(
        Some(vec![Matcher::Domain("example.com".to_owned())]),
        parse_matcher_rule("Host(`example.com`)")
    );
}

#[test]
fn parse_multiple() {
    assert_eq!(
        Some(vec![
            Matcher::Domain("example.com".to_owned()),
            Matcher::PathPrefix("/api/".to_owned())
        ]),
        parse_matcher_rule("Host(`example.com`) && PathPrefix(`/api/`)")
    );
}
