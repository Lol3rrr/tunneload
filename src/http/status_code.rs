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

    pub fn serialize(&self) -> [u8; 3] {
        match *self {
            Self::Continue => [b'1', b'0', b'0'],
            Self::SwitchingProtocols => [b'1', b'0', b'1'],
            Self::OK => [b'2', b'0', b'0'],
            Self::Created => [b'2', b'0', b'1'],
            Self::Accepted => [b'2', b'0', b'2'],
            Self::NonAuthoritativeInformation => [b'2', b'0', b'3'],
            Self::NoContent => [b'2', b'0', b'4'],
            Self::ResetContent => [b'2', b'0', b'5'],
            Self::PartialContent => [b'2', b'0', b'6'],
            Self::MultipleChoices => [b'3', b'0', b'0'],
            Self::MovedPermanently => [b'3', b'0', b'1'],
            Self::Found => [b'3', b'0', b'2'],
            Self::SeeOther => [b'3', b'0', b'3'],
            Self::NotModified => [b'3', b'0', b'4'],
            Self::UseProxy => [b'3', b'0', b'5'],
            Self::TemporaryRedirect => [b'3', b'0', b'7'],
            Self::BadRequest => [b'4', b'0', b'0'],
            Self::Unauthorized => [b'4', b'0', b'1'],
            Self::PaymentRequired => [b'4', b'0', b'2'],
            Self::Forbidden => [b'4', b'0', b'3'],
            Self::NotFound => [b'4', b'0', b'4'],
            Self::MethodNotAllowed => [b'4', b'0', b'5'],
            Self::NotAcceptable => [b'4', b'0', b'6'],
            Self::ProxyAuthenticationRequired => [b'4', b'0', b'7'],
            Self::RequestTimeOut => [b'4', b'0', b'8'],
            Self::Conflict => [b'4', b'0', b'9'],
            Self::Gone => [b'4', b'1', b'0'],
            Self::LengthRequired => [b'4', b'1', b'1'],
            Self::PreconditionFailed => [b'4', b'1', b'2'],
            Self::RequestEntityTooLarge => [b'4', b'1', b'3'],
            Self::RequestURITooLarge => [b'4', b'1', b'4'],
            Self::UnsupportedMediaType => [b'4', b'1', b'5'],
            Self::RequestedRangeNotSatisfiable => [b'4', b'1', b'6'],
            Self::ExpectationFailed => [b'4', b'1', b'7'],
            Self::ImATeapot => [b'4', b'1', b'8'],
            Self::InternalServerError => [b'5', b'0', b'0'],
            Self::NotImplemented => [b'5', b'0', b'1'],
            Self::BadGateway => [b'5', b'0', b'2'],
            Self::ServiceUnavailable => [b'5', b'0', b'3'],
            Self::GatewayTimeout => [b'5', b'0', b'4'],
            Self::HTTPVersionNotSupported => [b'5', b'0', b'5'],
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
    assert_eq!([b'1', b'0', b'0'], StatusCode::Continue.serialize());
    assert_eq!(
        [b'1', b'0', b'1'],
        StatusCode::SwitchingProtocols.serialize()
    );
    assert_eq!([b'2', b'0', b'0'], StatusCode::OK.serialize());
    assert_eq!([b'2', b'0', b'1'], StatusCode::Created.serialize());
    assert_eq!([b'2', b'0', b'2'], StatusCode::Accepted.serialize());
    assert_eq!(
        [b'2', b'0', b'3'],
        StatusCode::NonAuthoritativeInformation.serialize()
    );
    assert_eq!([b'2', b'0', b'4'], StatusCode::NoContent.serialize());
    assert_eq!([b'2', b'0', b'5'], StatusCode::ResetContent.serialize());
    assert_eq!([b'2', b'0', b'6'], StatusCode::PartialContent.serialize());

    assert_eq!([b'3', b'0', b'0'], StatusCode::MultipleChoices.serialize());
    assert_eq!([b'3', b'0', b'1'], StatusCode::MovedPermanently.serialize());
    assert_eq!([b'3', b'0', b'2'], StatusCode::Found.serialize());
    assert_eq!([b'3', b'0', b'3'], StatusCode::SeeOther.serialize());
    assert_eq!([b'3', b'0', b'4'], StatusCode::NotModified.serialize());
    assert_eq!([b'3', b'0', b'5'], StatusCode::UseProxy.serialize());
    assert_eq!(
        [b'3', b'0', b'7'],
        StatusCode::TemporaryRedirect.serialize()
    );

    assert_eq!([b'4', b'0', b'0'], StatusCode::BadRequest.serialize());
    assert_eq!([b'4', b'0', b'1'], StatusCode::Unauthorized.serialize());
    assert_eq!([b'4', b'0', b'2'], StatusCode::PaymentRequired.serialize());
    assert_eq!([b'4', b'0', b'3'], StatusCode::Forbidden.serialize());
    assert_eq!([b'4', b'0', b'4'], StatusCode::NotFound.serialize());
    assert_eq!([b'4', b'0', b'5'], StatusCode::MethodNotAllowed.serialize());
    assert_eq!([b'4', b'0', b'6'], StatusCode::NotAcceptable.serialize());
    assert_eq!(
        [b'4', b'0', b'7'],
        StatusCode::ProxyAuthenticationRequired.serialize()
    );
    assert_eq!([b'4', b'0', b'8'], StatusCode::RequestTimeOut.serialize());
    assert_eq!([b'4', b'0', b'9'], StatusCode::Conflict.serialize());
    assert_eq!([b'4', b'1', b'0'], StatusCode::Gone.serialize());
    assert_eq!([b'4', b'1', b'1'], StatusCode::LengthRequired.serialize());
    assert_eq!(
        [b'4', b'1', b'2'],
        StatusCode::PreconditionFailed.serialize()
    );
    assert_eq!(
        [b'4', b'1', b'3'],
        StatusCode::RequestEntityTooLarge.serialize()
    );
    assert_eq!(
        [b'4', b'1', b'4'],
        StatusCode::RequestURITooLarge.serialize()
    );
    assert_eq!(
        [b'4', b'1', b'5'],
        StatusCode::UnsupportedMediaType.serialize()
    );
    assert_eq!(
        [b'4', b'1', b'6'],
        StatusCode::RequestedRangeNotSatisfiable.serialize()
    );
    assert_eq!(
        [b'4', b'1', b'7'],
        StatusCode::ExpectationFailed.serialize()
    );
    assert_eq!([b'4', b'1', b'8'], StatusCode::ImATeapot.serialize());

    assert_eq!(
        [b'5', b'0', b'0'],
        StatusCode::InternalServerError.serialize()
    );
    assert_eq!([b'5', b'0', b'1'], StatusCode::NotImplemented.serialize());
    assert_eq!([b'5', b'0', b'2'], StatusCode::BadGateway.serialize());
    assert_eq!(
        [b'5', b'0', b'3'],
        StatusCode::ServiceUnavailable.serialize()
    );
    assert_eq!([b'5', b'0', b'4'], StatusCode::GatewayTimeout.serialize());
    assert_eq!(
        [b'5', b'0', b'5'],
        StatusCode::HTTPVersionNotSupported.serialize()
    );
}
