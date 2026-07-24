// Copyright 2026 Oxide Computer Company

//! HTTP statuses constrained to one status-code class.

use http::StatusCode;

/// An HTTP status whose hundreds digit is `CLASS`.
///
/// Generated response enums use this type for `OpenAPI` response ranges such as
/// `4XX`. This moves the range check to the point where an implementor chooses
/// the status, so converting a typed response into an HTTP response is
/// infallible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClassStatus<const CLASS: u16>(StatusCode);

impl<const CLASS: u16> ClassStatus<CLASS> {
    /// Constrains `status` to `CLASS`, returning [`None`] for another class.
    #[must_use]
    pub const fn new(status: StatusCode) -> Option<Self> {
        if status.as_u16() / 100 == CLASS {
            Some(Self(status))
        } else {
            None
        }
    }

    /// Returns the constrained HTTP status.
    #[must_use]
    pub const fn get(self) -> StatusCode {
        self.0
    }
}

macro_rules! class_statuses {
    ($class:literal { $($(#[$docs:meta])* $name:ident),+ $(,)? }) => {
        impl ClassStatus<$class> {
            $(
                $(#[$docs])*
                pub const $name: Self = Self(StatusCode::$name);
            )+
        }
    };
}

class_statuses!(1 {
    /// `100 Continue`.
    CONTINUE,
    /// `101 Switching Protocols`.
    SWITCHING_PROTOCOLS,
    /// `102 Processing`.
    PROCESSING,
    /// `103 Early Hints`.
    EARLY_HINTS,
});

class_statuses!(2 {
    /// `200 OK`.
    OK,
    /// `201 Created`.
    CREATED,
    /// `202 Accepted`.
    ACCEPTED,
    /// `203 Non-Authoritative Information`.
    NON_AUTHORITATIVE_INFORMATION,
    /// `204 No Content`.
    NO_CONTENT,
    /// `205 Reset Content`.
    RESET_CONTENT,
    /// `206 Partial Content`.
    PARTIAL_CONTENT,
    /// `207 Multi-Status`.
    MULTI_STATUS,
    /// `208 Already Reported`.
    ALREADY_REPORTED,
    /// `226 IM Used`.
    IM_USED,
});

class_statuses!(3 {
    /// `300 Multiple Choices`.
    MULTIPLE_CHOICES,
    /// `301 Moved Permanently`.
    MOVED_PERMANENTLY,
    /// `302 Found`.
    FOUND,
    /// `303 See Other`.
    SEE_OTHER,
    /// `304 Not Modified`.
    NOT_MODIFIED,
    /// `305 Use Proxy`.
    USE_PROXY,
    /// `307 Temporary Redirect`.
    TEMPORARY_REDIRECT,
    /// `308 Permanent Redirect`.
    PERMANENT_REDIRECT,
});

class_statuses!(4 {
    /// `400 Bad Request`.
    BAD_REQUEST,
    /// `401 Unauthorized`.
    UNAUTHORIZED,
    /// `402 Payment Required`.
    PAYMENT_REQUIRED,
    /// `403 Forbidden`.
    FORBIDDEN,
    /// `404 Not Found`.
    NOT_FOUND,
    /// `405 Method Not Allowed`.
    METHOD_NOT_ALLOWED,
    /// `406 Not Acceptable`.
    NOT_ACCEPTABLE,
    /// `407 Proxy Authentication Required`.
    PROXY_AUTHENTICATION_REQUIRED,
    /// `408 Request Timeout`.
    REQUEST_TIMEOUT,
    /// `409 Conflict`.
    CONFLICT,
    /// `410 Gone`.
    GONE,
    /// `411 Length Required`.
    LENGTH_REQUIRED,
    /// `412 Precondition Failed`.
    PRECONDITION_FAILED,
    /// `413 Payload Too Large`.
    PAYLOAD_TOO_LARGE,
    /// `414 URI Too Long`.
    URI_TOO_LONG,
    /// `415 Unsupported Media Type`.
    UNSUPPORTED_MEDIA_TYPE,
    /// `416 Range Not Satisfiable`.
    RANGE_NOT_SATISFIABLE,
    /// `417 Expectation Failed`.
    EXPECTATION_FAILED,
    /// `418 I'm a teapot`.
    IM_A_TEAPOT,
    /// `421 Misdirected Request`.
    MISDIRECTED_REQUEST,
    /// `422 Unprocessable Entity`.
    UNPROCESSABLE_ENTITY,
    /// `423 Locked`.
    LOCKED,
    /// `424 Failed Dependency`.
    FAILED_DEPENDENCY,
    /// `425 Too Early`.
    TOO_EARLY,
    /// `426 Upgrade Required`.
    UPGRADE_REQUIRED,
    /// `428 Precondition Required`.
    PRECONDITION_REQUIRED,
    /// `429 Too Many Requests`.
    TOO_MANY_REQUESTS,
    /// `431 Request Header Fields Too Large`.
    REQUEST_HEADER_FIELDS_TOO_LARGE,
    /// `451 Unavailable For Legal Reasons`.
    UNAVAILABLE_FOR_LEGAL_REASONS,
});

class_statuses!(5 {
    /// `500 Internal Server Error`.
    INTERNAL_SERVER_ERROR,
    /// `501 Not Implemented`.
    NOT_IMPLEMENTED,
    /// `502 Bad Gateway`.
    BAD_GATEWAY,
    /// `503 Service Unavailable`.
    SERVICE_UNAVAILABLE,
    /// `504 Gateway Timeout`.
    GATEWAY_TIMEOUT,
    /// `505 HTTP Version Not Supported`.
    HTTP_VERSION_NOT_SUPPORTED,
    /// `506 Variant Also Negotiates`.
    VARIANT_ALSO_NEGOTIATES,
    /// `507 Insufficient Storage`.
    INSUFFICIENT_STORAGE,
    /// `508 Loop Detected`.
    LOOP_DETECTED,
    /// `510 Not Extended`.
    NOT_EXTENDED,
    /// `511 Network Authentication Required`.
    NETWORK_AUTHENTICATION_REQUIRED,
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_only_the_requested_class() {
        assert_eq!(
            ClassStatus::<4>::new(StatusCode::NOT_FOUND),
            Some(ClassStatus::<4>::NOT_FOUND)
        );
        assert_eq!(ClassStatus::<4>::new(StatusCode::OK), None);
    }

    #[test]
    fn standard_constants_return_their_http_status() {
        assert_eq!(ClassStatus::<2>::CREATED.get(), StatusCode::CREATED);
        assert_eq!(ClassStatus::<4>::CONFLICT.get(), StatusCode::CONFLICT);
        assert_eq!(
            ClassStatus::<5>::SERVICE_UNAVAILABLE.get(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
