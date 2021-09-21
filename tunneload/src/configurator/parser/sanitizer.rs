#[derive(Debug, PartialEq)]
pub enum NameError {
    UnknownGroup(String),
}

#[derive(Debug, PartialEq)]
pub enum Group {
    Common,
    Plugin,
}

/// This is used to turn the given raw Name into the actual name plus the Group
/// it belongs to
pub fn get_name_group(name: &str) -> Result<(&str, Group), NameError> {
    let split_index = match name.find('@') {
        Some(i) => i,
        None => return Ok((name, Group::Common)),
    };

    let (name, group_name) = name.split_at(split_index);

    let group = match &group_name[1..] {
        "plugin" => Group::Plugin,
        name => return Err(NameError::UnknownGroup(name.to_owned())),
    };

    Ok((name, group))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_name() {
        let input = "test-name";
        let expected = Ok(("test-name", Group::Common));

        let result = get_name_group(input);

        assert_eq!(expected, result);
    }

    #[test]
    fn plugin_name() {
        let input = "test-name@plugin";
        let expected = Ok(("test-name", Group::Plugin));

        let result = get_name_group(input);

        assert_eq!(expected, result);
    }

    #[test]
    fn unknown_group() {
        let input = "test-name@other";
        let expected = Err(NameError::UnknownGroup("other".to_owned()));

        let result = get_name_group(input);

        assert_eq!(expected, result);
    }
}
