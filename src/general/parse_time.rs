pub fn parse_time(raw: &str) -> Option<std::time::Duration> {
    if raw.len() < 2 {
        return None;
    }

    let indicator_index = raw.len() - 1;
    let indicator = raw.as_bytes()[indicator_index];
    let duration = &raw[..indicator_index];
    let value = match duration.parse::<u64>() {
        Ok(v) => v,
        Err(_) => {
            return None;
        }
    };

    match indicator {
        b's' => Some(std::time::Duration::from_secs(value)),
        b'm' => Some(std::time::Duration::from_secs(value * 60)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seconds() {
        assert_eq!(Some(std::time::Duration::from_secs(25)), parse_time("25s"));
    }
    #[test]
    fn test_whole_minutes() {
        assert_eq!(
            Some(std::time::Duration::from_secs(2 * 60)),
            parse_time("2m")
        );
    }

    #[test]
    fn test_invalid_non_whole_number() {
        assert_eq!(None, parse_time("1.5m"));
    }
    #[test]
    fn test_invalid_empty_input() {
        assert_eq!(None, parse_time(""));
    }
    #[test]
    fn test_invalid_no_format() {
        assert_eq!(None, parse_time("2"));
    }
}
