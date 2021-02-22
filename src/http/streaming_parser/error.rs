pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    MissingMethod,
    MissingPath,
    MissingProtocol,
    MissingHeaders,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::MissingMethod => write!(f, "Missing Method"),
            Self::MissingPath => write!(f, "Missing Path"),
            Self::MissingProtocol => write!(f, "Missing Protocol"),
            Self::MissingHeaders => write!(f, "Missing Headers"),
        }
    }
}
