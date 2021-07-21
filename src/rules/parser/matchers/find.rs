#[derive(Debug, PartialEq)]
pub enum Combinator {
    And,
    Or,
}

/// Searches for an "&&" or "||" that seperates two parts on the
/// highest level
pub fn split(raw: &str) -> Option<(&str, &str, Combinator)> {
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
                } else {
                    return None;
                }
            }
            b'|' if open_brackets == 0 => {
                if bytes.get(index + 1) == Some(&b'|') {
                    return Some((&raw[0..index], &raw[index + 2..raw.len()], Combinator::Or));
                } else {
                    return None;
                }
            }
            _ => {}
        };
    }

    None
}

pub fn inner(raw: &str) -> Option<(&str, &str)> {
    let mut open_brackets = 0;

    let mut start = 0;

    for (index, tmp_char) in raw.as_bytes().iter().enumerate() {
        match tmp_char {
            b'(' => {
                if open_brackets == 0 {
                    start = index + 1;
                }
                open_brackets += 1;
            }
            b')' if open_brackets == 1 => {
                let prefix = &raw[..start - 1];
                let inner = &raw[start..index];
                return Some((prefix, inner));
            }
            b')' => {
                open_brackets -= 1;
            }
            _ => {}
        };
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_and_valid() {
        let input = "first&&second";
        let expected = Some(("first", "second", Combinator::And));

        assert_eq!(expected, split(input));
    }
    #[test]
    fn split_and_invalid() {
        let input = "first&second";
        let expected = None;

        assert_eq!(expected, split(input));
    }
    #[test]
    fn split_and_valid_subs() {
        let input = "(first || first_other)&&second";
        let expected = Some(("(first || first_other)", "second", Combinator::And));

        assert_eq!(expected, split(input));
    }
    #[test]
    fn split_and_invalid_subs() {
        let input = "(first || first_other &&second";
        let expected = None;

        assert_eq!(expected, split(input));
    }

    #[test]
    fn split_or_valid() {
        let input = "first||second";
        let expected = Some(("first", "second", Combinator::Or));

        assert_eq!(expected, split(input));
    }
    #[test]
    fn split_or_invalid() {
        let input = "first|second";
        let expected = None;

        assert_eq!(expected, split(input));
    }
}
