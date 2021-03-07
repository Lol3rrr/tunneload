use crate::rules::Matcher;

#[derive(Debug)]
enum Combinator {
    And,
    Or,
}

/// Searches for an "&&" or "||" that seperates two parts on the
/// highest level
fn find_split(raw: &str) -> Option<(&str, &str, Combinator)> {
    let mut open_brackets = 0;
    let bytes = raw.as_bytes();
    let raw_iter = raw.as_bytes().iter();
    let enumator = raw_iter.enumerate();
    for (index, tmp_char) in enumator {
        match tmp_char {
            b'(' => {
                open_brackets += 1;
            }
            b')' => {
                open_brackets -= 1;
            }
            b'&' if open_brackets == 0 => {
                if bytes.get(index + 1) == Some(&b'&') {
                    return Some((&raw[0..index], &raw[index + 2..raw.len()], Combinator::And));
                }
            }
            b'|' if open_brackets == 0 => {
                if bytes.get(index + 1) == Some(&b'|') {
                    return Some((&raw[0..index], &raw[index + 2..raw.len()], Combinator::Or));
                }
            }
            _ => {}
        };
    }

    None
}

/// Parses a raw String that defines matchers
///
///
pub fn parse_matchers(raw: &str) -> Option<Matcher> {
    let cleaned = raw.replace(" ", "");
    let raw = cleaned.as_str();

    match find_split(raw) {
        Some(middle) => {
            let first_part = middle.0;
            let first_part = match first_part.strip_prefix("(") {
                Some(n) => n,
                None => first_part,
            };
            let first_part = match first_part.strip_suffix(")") {
                Some(n) => n,
                None => first_part,
            };
            let first = match parse_matchers(first_part) {
                Some(v) => v,
                None => {
                    return None;
                }
            };

            let second_part = middle.1;
            let second_part = match second_part.strip_prefix("(") {
                Some(n) => n,
                None => second_part,
            };
            let second_part = match second_part.strip_suffix(")") {
                Some(n) => n,
                None => second_part,
            };
            let second = match parse_matchers(second_part) {
                Some(v) => v,
                None => {
                    return None;
                }
            };

            let combinator = middle.2;

            match combinator {
                Combinator::And => Some(Matcher::And(vec![first, second])),
                Combinator::Or => Some(Matcher::Or(vec![first, second])),
            }
        }
        None => {
            let key_end = match raw.find('(') {
                Some(k) => k,
                None => {
                    return None;
                }
            };
            let (key, rest) = raw.split_at(key_end);

            let inner = match rest.split('`').nth(1) {
                Some(i) => i,
                None => {
                    return None;
                }
            };

            match key {
                "Host" => Some(Matcher::Domain(inner.to_owned())),
                "PathPrefix" => Some(Matcher::PathPrefix(inner.to_owned())),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single() {
        assert_eq!(
            Some(Matcher::Domain("example.com".to_owned())),
            parse_matchers("Host(`example.com`)")
        );
    }

    #[test]
    fn parse_two_with_and() {
        assert_eq!(
            Some(Matcher::And(vec![
                Matcher::Domain("example.com".to_owned()),
                Matcher::PathPrefix("/api/".to_owned())
            ])),
            parse_matchers("Host(`example.com`) && PathPrefix(`/api/`)")
        );
    }

    #[test]
    fn parse_two_with_or() {
        assert_eq!(
            Some(Matcher::Or(vec![
                Matcher::Domain("example.com".to_owned()),
                Matcher::PathPrefix("/api/".to_owned())
            ])),
            parse_matchers("Host(`example.com`) || PathPrefix(`/api/`)")
        );
    }

    #[test]
    fn parse_single_and_two_or_nested() {
        assert_eq!(
            Some(Matcher::And(vec![
                Matcher::Domain("example.com".to_owned()),
                Matcher::Or(vec![
                    Matcher::PathPrefix("/api/".to_owned()),
                    Matcher::PathPrefix("/dashboard/".to_owned())
                ]),
            ])),
            parse_matchers(
                "Host(`example.com`) && ( PathPrefix(`/api/`) || PathPrefix(`/dashboard/`) )"
            )
        );
    }

    #[test]
    fn parse_two_or_and_two_or_nested() {
        assert_eq!(
        Some(Matcher::And(vec![
            Matcher::Or(vec![
                Matcher::Domain("example.com".to_owned()),
                Matcher::Domain("example.net".to_owned())
            ]),
            Matcher::Or(vec![
                Matcher::PathPrefix("/api/".to_owned()),
                Matcher::PathPrefix("/dashboard/".to_owned())
            ]),
        ])),
        parse_matchers(
            "(Host(`example.com`) || Host(`example.net`)) && (PathPrefix(`/api/`) || PathPrefix(`/dashboard/`))"
        )
    );
    }

    #[test]
    fn parse_two_and_and_two_or_nested() {
        assert_eq!(
        Some(Matcher::And(vec![
            Matcher::And(vec![
                Matcher::PathPrefix("/api/".to_owned()),
                Matcher::PathPrefix("/api/test/".to_owned())
            ]),
            Matcher::Or(vec![
                Matcher::Domain("example.com".to_owned()),
                Matcher::Domain("example.net".to_owned())
            ]),
        ])),
        parse_matchers(
            "(PathPrefix(`/api/`) && PathPrefix(`/api/test/`)) && (Host(`example.com`) || Host(`example.net`))"
        )
    );
    }

    #[test]
    fn parse_invalid_pair_first_missing() {
        assert_eq!(None, parse_matchers("PathPrefix(`/api/`) &&"));
    }
    #[test]
    fn parse_invalid_pair_second_missing() {
        assert_eq!(None, parse_matchers("&& PathPrefix(`/api/`)"));
    }

    #[test]
    fn parse_invalid_missing_closing_bracket() {
        assert_eq!(
            None,
            parse_matchers(
                "Domain(`example.net`) && (PathPrefix(`/api/`) || PathPrefix(`/dashboard/`)"
            )
        );
    }
}
