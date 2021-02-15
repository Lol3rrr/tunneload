#[derive(Debug, PartialEq)]
pub enum StatusCode {
    Continue,
    SwitchingProtocols,
    OK,
    Created,
    Accepted,
    NonAuthoritativeInformation,
    NoContent,
    ResetContent,
    PartialContent,
    MultipleChoices,
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    UseProxy,
    TemporaryRedirect,
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeOut,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    RequestEntityTooLarge,
    RequestURITooLarge,
    UnsupportedMediaType,
    RequestedRangeNotSatisfiable,
    ExpectationFailed,
    ImATeapot,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HTTPVersionNotSupported,
}

impl StatusCode {
    /// Parses the Raw Response Status-Code to the enum
    pub fn parse(raw: &str) -> Option<Self> {
        if raw.len() < 3 {
            return None;
        }

        let key = &raw[0..3];

        match key {
            "100" => Some(StatusCode::Continue),
            "101" => Some(StatusCode::SwitchingProtocols),
            "200" => Some(StatusCode::OK),
            "201" => Some(StatusCode::Created),
            "202" => Some(StatusCode::Accepted),
            "203" => Some(StatusCode::NonAuthoritativeInformation),
            "204" => Some(StatusCode::NoContent),
            "205" => Some(StatusCode::ResetContent),
            "206" => Some(StatusCode::PartialContent),
            "300" => Some(StatusCode::MultipleChoices),
            "301" => Some(StatusCode::MovedPermanently),
            "302" => Some(StatusCode::Found),
            "303" => Some(StatusCode::SeeOther),
            "304" => Some(StatusCode::NotModified),
            "305" => Some(StatusCode::UseProxy),
            "307" => Some(StatusCode::TemporaryRedirect),
            "400" => Some(StatusCode::BadRequest),
            "401" => Some(StatusCode::Unauthorized),
            "402" => Some(StatusCode::PaymentRequired),
            "403" => Some(StatusCode::Forbidden),
            "404" => Some(StatusCode::NotFound),
            "405" => Some(StatusCode::MethodNotAllowed),
            "406" => Some(StatusCode::NotAcceptable),
            "407" => Some(StatusCode::ProxyAuthenticationRequired),
            "408" => Some(StatusCode::RequestTimeOut),
            "409" => Some(StatusCode::Conflict),
            "410" => Some(StatusCode::Gone),
            "411" => Some(StatusCode::LengthRequired),
            "412" => Some(StatusCode::PreconditionFailed),
            "413" => Some(StatusCode::RequestEntityTooLarge),
            "414" => Some(StatusCode::RequestURITooLarge),
            "415" => Some(StatusCode::UnsupportedMediaType),
            "416" => Some(StatusCode::RequestedRangeNotSatisfiable),
            "417" => Some(StatusCode::ExpectationFailed),
            "418" => Some(StatusCode::ImATeapot),
            "500" => Some(StatusCode::InternalServerError),
            "501" => Some(StatusCode::NotImplemented),
            "502" => Some(StatusCode::BadGateway),
            "503" => Some(StatusCode::ServiceUnavailable),
            "504" => Some(StatusCode::GatewayTimeout),
            "505" => Some(StatusCode::HTTPVersionNotSupported),
            _ => None,
        }
    }

    pub fn serialize(&self) -> String {
        match *self {
            Self::Continue => "100 Continue".to_owned(),
            Self::SwitchingProtocols => "101 Switching Protocols".to_owned(),
            Self::OK => "200 OK".to_owned(),
            Self::Created => "201 Created".to_owned(),
            Self::Accepted => "202 Accepted".to_owned(),
            Self::NonAuthoritativeInformation => "203 Non-Authoritative Information".to_owned(),
            Self::NoContent => "204 No Content".to_owned(),
            Self::ResetContent => "205 Reset Content".to_owned(),
            Self::PartialContent => "206 Partial Content".to_owned(),
            Self::MultipleChoices => "300 Multiple Choices".to_owned(),
            Self::MovedPermanently => "301 Moved Permanently".to_owned(),
            Self::Found => "302 Found".to_owned(),
            Self::SeeOther => "303 See Other".to_owned(),
            Self::NotModified => "304 Not Modified".to_owned(),
            Self::UseProxy => "305 Use Proxy".to_owned(),
            Self::TemporaryRedirect => "307 Temporary Redirect".to_owned(),
            Self::BadRequest => "400 Bad Request".to_owned(),
            Self::Unauthorized => "401 Unauthorized".to_owned(),
            Self::PaymentRequired => "402 Payment Required".to_owned(),
            Self::Forbidden => "403 Forbidden".to_owned(),
            Self::NotFound => "404 Not Found".to_owned(),
            Self::MethodNotAllowed => "405 Method Not Allowed".to_owned(),
            Self::NotAcceptable => "406 Not Acceptable".to_owned(),
            Self::ProxyAuthenticationRequired => "407 Proxy Authentication Required".to_owned(),
            Self::RequestTimeOut => "408 Request Time-out".to_owned(),
            Self::Conflict => "409 Conflict".to_owned(),
            Self::Gone => "410 Gone".to_owned(),
            Self::LengthRequired => "411 Length Required".to_owned(),
            Self::PreconditionFailed => "412 Precondition Failed".to_owned(),
            Self::RequestEntityTooLarge => "413 Request Entity Too Large".to_owned(),
            Self::RequestURITooLarge => "414 Request-URI Too Large".to_owned(),
            Self::UnsupportedMediaType => "415 Unsupported Media Type".to_owned(),
            Self::RequestedRangeNotSatisfiable => "416 Requested Range Not Satisfiable".to_owned(),
            Self::ExpectationFailed => "417 Expectation Failed".to_owned(),
            Self::ImATeapot => "418 I'm a Teapot".to_owned(),
            Self::InternalServerError => "500 Internal Server Error".to_owned(),
            Self::NotImplemented => "501 Not Implemented".to_owned(),
            Self::BadGateway => "502 Bad Gateway".to_owned(),
            Self::ServiceUnavailable => "503 Service Unavailable".to_owned(),
            Self::GatewayTimeout => "504 Gateway Time-out".to_owned(),
            Self::HTTPVersionNotSupported => "505 HTTP Version Not Supported".to_owned(),
        }
    }
}

#[test]
fn parse_invalid() {
    assert_eq!(None, StatusCode::parse("1"));
    assert_eq!(None, StatusCode::parse("123"));
}

#[test]
fn parse_all() {
    assert_eq!(Some(StatusCode::Continue), StatusCode::parse("100"));
    assert_eq!(
        Some(StatusCode::SwitchingProtocols),
        StatusCode::parse("101")
    );
    assert_eq!(Some(StatusCode::OK), StatusCode::parse("200"));
    assert_eq!(Some(StatusCode::Created), StatusCode::parse("201"));
    assert_eq!(Some(StatusCode::Accepted), StatusCode::parse("202"));
    assert_eq!(
        Some(StatusCode::NonAuthoritativeInformation),
        StatusCode::parse("203")
    );
    assert_eq!(Some(StatusCode::NoContent), StatusCode::parse("204"));
    assert_eq!(Some(StatusCode::ResetContent), StatusCode::parse("205"));
    assert_eq!(Some(StatusCode::PartialContent), StatusCode::parse("206"));
    assert_eq!(Some(StatusCode::MultipleChoices), StatusCode::parse("300"));
    assert_eq!(Some(StatusCode::MovedPermanently), StatusCode::parse("301"));
    assert_eq!(Some(StatusCode::Found), StatusCode::parse("302"));
    assert_eq!(Some(StatusCode::SeeOther), StatusCode::parse("303"));
    assert_eq!(Some(StatusCode::NotModified), StatusCode::parse("304"));
    assert_eq!(Some(StatusCode::UseProxy), StatusCode::parse("305"));
    assert_eq!(
        Some(StatusCode::TemporaryRedirect),
        StatusCode::parse("307")
    );
    assert_eq!(Some(StatusCode::BadRequest), StatusCode::parse("400"));
    assert_eq!(Some(StatusCode::Unauthorized), StatusCode::parse("401"));
    assert_eq!(Some(StatusCode::PaymentRequired), StatusCode::parse("402"));
    assert_eq!(Some(StatusCode::Forbidden), StatusCode::parse("403"));
    assert_eq!(Some(StatusCode::NotFound), StatusCode::parse("404"));
    assert_eq!(Some(StatusCode::MethodNotAllowed), StatusCode::parse("405"));
    assert_eq!(Some(StatusCode::NotAcceptable), StatusCode::parse("406"));
    assert_eq!(
        Some(StatusCode::ProxyAuthenticationRequired),
        StatusCode::parse("407")
    );
    assert_eq!(Some(StatusCode::RequestTimeOut), StatusCode::parse("408"));
    assert_eq!(Some(StatusCode::Conflict), StatusCode::parse("409"));
    assert_eq!(Some(StatusCode::Gone), StatusCode::parse("410"));
    assert_eq!(Some(StatusCode::LengthRequired), StatusCode::parse("411"));
    assert_eq!(
        Some(StatusCode::PreconditionFailed),
        StatusCode::parse("412")
    );
    assert_eq!(
        Some(StatusCode::RequestEntityTooLarge),
        StatusCode::parse("413")
    );
    assert_eq!(
        Some(StatusCode::RequestURITooLarge),
        StatusCode::parse("414")
    );
    assert_eq!(
        Some(StatusCode::UnsupportedMediaType),
        StatusCode::parse("415")
    );
    assert_eq!(
        Some(StatusCode::RequestedRangeNotSatisfiable),
        StatusCode::parse("416")
    );
    assert_eq!(
        Some(StatusCode::ExpectationFailed),
        StatusCode::parse("417")
    );
    assert_eq!(Some(StatusCode::ImATeapot), StatusCode::parse("418"));
    assert_eq!(
        Some(StatusCode::InternalServerError),
        StatusCode::parse("500")
    );
    assert_eq!(Some(StatusCode::NotImplemented), StatusCode::parse("501"));
    assert_eq!(Some(StatusCode::BadGateway), StatusCode::parse("502"));
    assert_eq!(
        Some(StatusCode::ServiceUnavailable),
        StatusCode::parse("503")
    );
    assert_eq!(Some(StatusCode::GatewayTimeout), StatusCode::parse("504"));
    assert_eq!(
        Some(StatusCode::HTTPVersionNotSupported),
        StatusCode::parse("505")
    );
}

#[test]
fn serialize() {
    assert_eq!("100 Continue".to_owned(), StatusCode::Continue.serialize());
    assert_eq!(
        "101 Switching Protocols".to_owned(),
        StatusCode::SwitchingProtocols.serialize()
    );
    assert_eq!("200 OK".to_owned(), StatusCode::OK.serialize());
    assert_eq!("201 Created".to_owned(), StatusCode::Created.serialize());
    assert_eq!("202 Accepted".to_owned(), StatusCode::Accepted.serialize());
    assert_eq!(
        "203 Non-Authoritative Information".to_owned(),
        StatusCode::NonAuthoritativeInformation.serialize()
    );
    assert_eq!(
        "204 No Content".to_owned(),
        StatusCode::NoContent.serialize()
    );
    assert_eq!(
        "205 Reset Content".to_owned(),
        StatusCode::ResetContent.serialize()
    );
    assert_eq!(
        "206 Partial Content".to_owned(),
        StatusCode::PartialContent.serialize()
    );

    assert_eq!(
        "300 Multiple Choices".to_owned(),
        StatusCode::MultipleChoices.serialize()
    );
    assert_eq!(
        "301 Moved Permanently".to_owned(),
        StatusCode::MovedPermanently.serialize()
    );
    assert_eq!("302 Found".to_owned(), StatusCode::Found.serialize());
    assert_eq!("303 See Other".to_owned(), StatusCode::SeeOther.serialize());
    assert_eq!(
        "304 Not Modified".to_owned(),
        StatusCode::NotModified.serialize()
    );
    assert_eq!("305 Use Proxy".to_owned(), StatusCode::UseProxy.serialize());
    assert_eq!(
        "307 Temporary Redirect".to_owned(),
        StatusCode::TemporaryRedirect.serialize()
    );

    assert_eq!(
        "400 Bad Request".to_owned(),
        StatusCode::BadRequest.serialize()
    );
    assert_eq!(
        "401 Unauthorized".to_owned(),
        StatusCode::Unauthorized.serialize()
    );
    assert_eq!(
        "402 Payment Required".to_owned(),
        StatusCode::PaymentRequired.serialize()
    );
    assert_eq!(
        "403 Forbidden".to_owned(),
        StatusCode::Forbidden.serialize()
    );
    assert_eq!("404 Not Found".to_owned(), StatusCode::NotFound.serialize());
    assert_eq!(
        "405 Method Not Allowed".to_owned(),
        StatusCode::MethodNotAllowed.serialize()
    );
    assert_eq!(
        "406 Not Acceptable".to_owned(),
        StatusCode::NotAcceptable.serialize()
    );
    assert_eq!(
        "407 Proxy Authentication Required".to_owned(),
        StatusCode::ProxyAuthenticationRequired.serialize()
    );
    assert_eq!(
        "408 Request Time-out".to_owned(),
        StatusCode::RequestTimeOut.serialize()
    );
    assert_eq!("409 Conflict".to_owned(), StatusCode::Conflict.serialize());
    assert_eq!("410 Gone".to_owned(), StatusCode::Gone.serialize());
    assert_eq!(
        "411 Length Required".to_owned(),
        StatusCode::LengthRequired.serialize()
    );
    assert_eq!(
        "412 Precondition Failed".to_owned(),
        StatusCode::PreconditionFailed.serialize()
    );
    assert_eq!(
        "413 Request Entity Too Large".to_owned(),
        StatusCode::RequestEntityTooLarge.serialize()
    );
    assert_eq!(
        "414 Request-URI Too Large".to_owned(),
        StatusCode::RequestURITooLarge.serialize()
    );
    assert_eq!(
        "415 Unsupported Media Type".to_owned(),
        StatusCode::UnsupportedMediaType.serialize()
    );
    assert_eq!(
        "416 Requested Range Not Satisfiable".to_owned(),
        StatusCode::RequestedRangeNotSatisfiable.serialize()
    );
    assert_eq!(
        "417 Expectation Failed".to_owned(),
        StatusCode::ExpectationFailed.serialize()
    );
    assert_eq!(
        "418 I'm a Teapot".to_owned(),
        StatusCode::ImATeapot.serialize()
    );

    assert_eq!(
        "500 Internal Server Error".to_owned(),
        StatusCode::InternalServerError.serialize()
    );
    assert_eq!(
        "501 Not Implemented".to_owned(),
        StatusCode::NotImplemented.serialize()
    );
    assert_eq!(
        "502 Bad Gateway".to_owned(),
        StatusCode::BadGateway.serialize()
    );
    assert_eq!(
        "503 Service Unavailable".to_owned(),
        StatusCode::ServiceUnavailable.serialize()
    );
    assert_eq!(
        "504 Gateway Time-out".to_owned(),
        StatusCode::GatewayTimeout.serialize()
    );
    assert_eq!(
        "505 HTTP Version Not Supported".to_owned(),
        StatusCode::HTTPVersionNotSupported.serialize()
    );
}
