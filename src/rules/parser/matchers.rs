use crate::rules::Matcher;

mod find;

/// Is simply used to get the inner part of something that is wrapped with
/// paranthese
fn inner_scope(raw: &str) -> &str {
    let tmp = match raw.strip_prefix('(') {
        Some(r) => r,
        None => {
            return raw;
        }
    };

    match tmp.strip_suffix(')') {
        Some(r) => r,
        None => raw,
    }
}

/// The Error returned when Parsing Matchers
#[derive(Debug, PartialEq)]
pub enum ParseMatcherError {
    /// The given String was in an Invalid-Format and could not be parsed as a
    /// valid Matcher
    Invalid,
    /// The specified Matcher is unknown to the Load-Balancer
    UnknownMatcher {
        /// The Unknown-Key that was specified
        key: String,
    },
}

/// Parses a raw String that defines matchers
pub fn parse_matchers(raw: &str) -> Result<Matcher, ParseMatcherError> {
    let cleaned = raw.replace(" ", "");
    let raw = cleaned.as_str();

    match find::split(raw) {
        Some((first_part, second_part, combinator)) => {
            let first_part = inner_scope(first_part);
            let first = match parse_matchers(first_part) {
                Ok(v) => v,
                Err(_) => {
                    return Err(ParseMatcherError::Invalid);
                }
            };

            let second_part = inner_scope(second_part);
            let second = match parse_matchers(second_part) {
                Ok(v) => v,
                Err(_) => {
                    return Err(ParseMatcherError::Invalid);
                }
            };

            match combinator {
                find::Combinator::And => Ok(Matcher::And(vec![first, second])),
                find::Combinator::Or => Ok(Matcher::Or(vec![first, second])),
            }
        }
        None => {
            let (key, raw_inner) = find::inner(raw).ok_or_else(|| ParseMatcherError::Invalid)?;

            let inner = raw_inner
                .split('`')
                .nth(1)
                .ok_or_else(|| ParseMatcherError::Invalid)?;

            match key {
                "Host" => Ok(Matcher::Domain(inner.to_owned())),
                "PathPrefix" => Ok(Matcher::PathPrefix(inner.to_owned())),
                _ => Err(ParseMatcherError::UnknownMatcher {
                    key: key.to_string(),
                }),
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
            Ok(Matcher::Domain("example.com".to_owned())),
            parse_matchers("Host(`example.com`)")
        );
    }

    #[test]
    fn parse_two_with_and() {
        assert_eq!(
            Ok(Matcher::And(vec![
                Matcher::Domain("example.com".to_owned()),
                Matcher::PathPrefix("/api/".to_owned())
            ])),
            parse_matchers("Host(`example.com`) && PathPrefix(`/api/`)")
        );
    }

    #[test]
    fn parse_two_with_or() {
        let input = "Host(`example.com`) || PathPrefix(`/api/`)";

        assert_eq!(
            Ok(Matcher::Or(vec![
                Matcher::Domain("example.com".to_owned()),
                Matcher::PathPrefix("/api/".to_owned())
            ])),
            parse_matchers(input)
        );
    }

    #[test]
    fn parse_single_and_two_or_nested() {
        let input = "Host(`example.com`) && ( PathPrefix(`/api/`) || PathPrefix(`/dashboard/`) )";

        assert_eq!(
            Ok(Matcher::And(vec![
                Matcher::Domain("example.com".to_owned()),
                Matcher::Or(vec![
                    Matcher::PathPrefix("/api/".to_owned()),
                    Matcher::PathPrefix("/dashboard/".to_owned())
                ]),
            ])),
            parse_matchers(input)
        );
    }

    #[test]
    fn parse_two_or_and_two_or_nested() {
        let input = "(Host(`example.com`) || Host(`example.net`)) && (PathPrefix(`/api/`) || PathPrefix(`/dashboard/`))";

        assert_eq!(
            Ok(Matcher::And(vec![
                Matcher::Or(vec![
                    Matcher::Domain("example.com".to_owned()),
                    Matcher::Domain("example.net".to_owned())
                ]),
                Matcher::Or(vec![
                    Matcher::PathPrefix("/api/".to_owned()),
                    Matcher::PathPrefix("/dashboard/".to_owned())
                ]),
            ])),
            parse_matchers(input)
        );
    }

    #[test]
    fn parse_two_and_and_two_or_nested() {
        let input = "(PathPrefix(`/api/`) && PathPrefix(`/api/test/`)) && (Host(`example.com`) || Host(`example.net`))";

        assert_eq!(
            Ok(Matcher::And(vec![
                Matcher::And(vec![
                    Matcher::PathPrefix("/api/".to_owned()),
                    Matcher::PathPrefix("/api/test/".to_owned())
                ]),
                Matcher::Or(vec![
                    Matcher::Domain("example.com".to_owned()),
                    Matcher::Domain("example.net".to_owned())
                ]),
            ])),
            parse_matchers(input)
        );
    }

    #[test]
    fn parse_invalid_pair_first_missing() {
        assert_eq!(
            Err(ParseMatcherError::Invalid),
            parse_matchers("PathPrefix(`/api/`) &&")
        );
    }
    #[test]
    fn parse_invalid_pair_second_missing() {
        assert_eq!(
            Err(ParseMatcherError::Invalid),
            parse_matchers("&& PathPrefix(`/api/`)")
        );
    }

    #[test]
    fn parse_invalid_missing_closing_bracket() {
        assert_eq!(
            Err(ParseMatcherError::Invalid),
            parse_matchers(
                "Domain(`example.net`) && (PathPrefix(`/api/`) || PathPrefix(`/dashboard/`)"
            )
        );
    }

    #[test]
    fn parse_invalid() {
        assert_eq!(
            Err(ParseMatcherError::Invalid),
            parse_matchers("Host(`example.net`")
        );

        assert_eq!(
            Err(ParseMatcherError::Invalid),
            parse_matchers("Host`example.net`)")
        );
    }
}
