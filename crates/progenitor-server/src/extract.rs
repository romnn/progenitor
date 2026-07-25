// Copyright 2026 Oxide Computer Company

//! Request extractors with controlled, service-rendered rejections.
//!
//! Generated routes use these custom extractors instead of the bare axum ones so
//! that an extraction failure carries a stable [`RejectionKind`] and status
//! rather than axum's backend-specific body. The generated service trait chooses
//! how to render that rejection. The query extractor also handles the
//! repeated-key array encoding the generated client emits (`?x=1&x=2`), which
//! axum's `serde_urlencoded`-based `Query` cannot.
//!
//! Extraction failures are intentionally service-wide, not an operation's typed
//! error: a generic extractor cannot construct an operation-specific response.
//! A generated client may therefore surface an undocumented rejection status as
//! an unexpected response; the real status and body remain observable via raw
//! HTTP.

use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::response::{IntoResponse, Response};
use http::request::Parts;
use http::{Extensions, HeaderMap, Method, StatusCode, Uri};
use serde::de::DeserializeOwned;

/// A machine-readable category for a request-level rejection.
///
/// The enum is non-exhaustive so new extraction surfaces can be categorized
/// without breaking service renderers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionKind {
    /// A request body could not be read or decoded.
    InvalidBody,
    /// The request body has a missing or unsupported media type.
    UnsupportedMediaType,
    /// Query parameters could not be decoded.
    InvalidQuery,
    /// Path parameters could not be decoded.
    InvalidPath,
    /// A required header is absent.
    MissingHeader,
    /// A header value could not be decoded.
    InvalidHeader,
    /// A required cookie is absent.
    MissingCookie,
    /// A cookie value could not be decoded.
    InvalidCookie,
    /// A required multipart part is absent.
    MissingPart,
    /// A multipart part could not be decoded.
    InvalidPart,
    /// The service encountered an unexpected internal failure.
    Internal,
}

/// A request-level failure with a stable category, status, and safe message.
///
/// [`IntoResponse`] preserves the runtime's plain-text default. Generated
/// services can override that representation through their rejection renderer.
#[derive(Debug, Clone)]
pub struct Rejection {
    status: StatusCode,
    kind: RejectionKind,
    message: String,
}

impl Rejection {
    /// Constructs a rejection with an explicit status and category.
    pub fn new(status: StatusCode, kind: RejectionKind, message: impl Into<String>) -> Self {
        Rejection {
            status,
            kind,
            message: message.into(),
        }
    }

    /// Constructs a body rejection while preserving the extractor's status.
    pub fn invalid_body(status: StatusCode, message: impl Into<String>) -> Self {
        Rejection::new(status, RejectionKind::InvalidBody, message)
    }

    /// Constructs a `415 Unsupported Media Type` rejection.
    pub fn unsupported_media_type(message: impl Into<String>) -> Self {
        Rejection::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            RejectionKind::UnsupportedMediaType,
            message,
        )
    }

    /// Constructs a `400 Bad Request` query rejection.
    pub fn invalid_query(message: impl Into<String>) -> Self {
        Rejection::new(
            StatusCode::BAD_REQUEST,
            RejectionKind::InvalidQuery,
            message,
        )
    }

    /// Constructs a path rejection while preserving the extractor's status.
    pub fn invalid_path(status: StatusCode, message: impl Into<String>) -> Self {
        Rejection::new(status, RejectionKind::InvalidPath, message)
    }

    /// Constructs a `400 Bad Request` rejection for an absent header.
    pub fn missing_header(message: impl Into<String>) -> Self {
        Rejection::new(
            StatusCode::BAD_REQUEST,
            RejectionKind::MissingHeader,
            message,
        )
    }

    /// Constructs a `400 Bad Request` rejection for an invalid header.
    pub fn invalid_header(message: impl Into<String>) -> Self {
        Rejection::new(
            StatusCode::BAD_REQUEST,
            RejectionKind::InvalidHeader,
            message,
        )
    }

    /// Constructs a `400 Bad Request` rejection for an absent cookie.
    pub fn missing_cookie(message: impl Into<String>) -> Self {
        Rejection::new(
            StatusCode::BAD_REQUEST,
            RejectionKind::MissingCookie,
            message,
        )
    }

    /// Constructs a `400 Bad Request` rejection for an invalid cookie.
    pub fn invalid_cookie(message: impl Into<String>) -> Self {
        Rejection::new(
            StatusCode::BAD_REQUEST,
            RejectionKind::InvalidCookie,
            message,
        )
    }

    /// Constructs a `400 Bad Request` rejection for an absent multipart part.
    pub fn missing_part(message: impl Into<String>) -> Self {
        Rejection::new(StatusCode::BAD_REQUEST, RejectionKind::MissingPart, message)
    }

    /// Constructs a `400 Bad Request` rejection for an invalid multipart part.
    pub fn invalid_part(message: impl Into<String>) -> Self {
        Rejection::new(StatusCode::BAD_REQUEST, RejectionKind::InvalidPart, message)
    }

    /// Constructs a generic `500 Internal Server Error` rejection.
    #[must_use]
    pub fn internal() -> Self {
        Rejection::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            RejectionKind::Internal,
            "Internal Server Error",
        )
    }

    /// Returns the HTTP status associated with this rejection.
    ///
    /// Generated server adapters restore this status after custom rendering.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns the machine-readable rejection category.
    #[must_use]
    pub fn kind(&self) -> RejectionKind {
        self.kind
    }

    /// Returns the safe, human-readable message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for Rejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
    }
}

impl std::error::Error for Rejection {}

impl IntoResponse for Rejection {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

/// The request metadata carried alongside the decoded message in
/// [`crate::Request`]: method, URI, headers, and extensions. Cloned out of the
/// request *before* the body extractor runs, so it composes with a body
/// extractor in the same handler.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    extensions: Extensions,
}

impl Metadata {
    /// The request method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// The request URI (use `.query()` for the raw query string).
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// The request headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// The request extensions (`ConnectInfo`, middleware-inserted values, ...).
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Metadata {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Metadata {
            method: parts.method.clone(),
            uri: parts.uri.clone(),
            headers: parts.headers.clone(),
            extensions: parts.extensions.clone(),
        })
    }
}

/// Typed path-parameter extractor (single value or a tuple in path order).
/// Wraps axum's `Path` but maps failures to a standardized [`Rejection`].
#[derive(Debug, Clone)]
pub struct Path<T>(pub T);

impl<T, S> FromRequestParts<S> for Path<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(axum::extract::Path(value)) => Ok(Path(value)),
            Err(rejection) => Err(Rejection::invalid_path(
                rejection.status(),
                "invalid path parameter",
            )),
        }
    }
}

/// Typed query extractor backed by `serde_html_form`, which (unlike axum's
/// `serde_urlencoded`-based `Query`) round-trips the repeated-key array encoding
/// the generated client emits (`?x=1&x=2`).
#[derive(Debug, Clone)]
pub struct Query<T>(pub T);

impl<T, S> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Rejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or("");
        match serde_html_form::from_str::<T>(query) {
            Ok(value) => Ok(Query(value)),
            Err(error) => Err(Rejection::invalid_query(format!(
                "invalid query string: {error}"
            ))),
        }
    }
}

/// Typed JSON body extractor. Wraps axum's `Json` but maps failures to a
/// standardized [`Rejection`] (preserving `415` for content-type mismatches).
#[derive(Debug, Clone)]
pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Rejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(Json(value)),
            Err(rejection) if rejection.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE => {
                Err(Rejection::unsupported_media_type("invalid JSON body"))
            }
            Err(rejection) => Err(Rejection::invalid_body(
                rejection.status(),
                "invalid JSON body",
            )),
        }
    }
}

/// `application/x-www-form-urlencoded` body extractor. Wraps axum's `Form`.
#[derive(Debug, Clone)]
pub struct Form<T>(pub T);

impl<T, S> FromRequest<S> for Form<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Rejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Form::<T>::from_request(req, state).await {
            Ok(axum::Form(value)) => Ok(Form(value)),
            Err(rejection) if rejection.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE => {
                Err(Rejection::unsupported_media_type("invalid form body"))
            }
            Err(rejection) => Err(Rejection::invalid_body(
                rejection.status(),
                "invalid form body",
            )),
        }
    }
}

/// Extracts a multipart body with runtime-standard rejection mapping.
#[derive(Debug)]
pub struct Multipart(pub(crate) axum::extract::Multipart);

impl<S> FromRequest<S> for Multipart
where
    S: Send + Sync,
{
    type Rejection = Rejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Multipart::from_request(req, state).await {
            Ok(value) => Ok(Multipart(value)),
            Err(rejection) => Err(Rejection::invalid_body(
                rejection.status(),
                "invalid multipart body",
            )),
        }
    }
}

/// Raw byte body extractor (octet-stream / other raw media types).
#[derive(Debug, Clone)]
pub struct Bytes(pub bytes::Bytes);

impl<S> FromRequest<S> for Bytes
where
    S: Send + Sync,
{
    type Rejection = Rejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match bytes::Bytes::from_request(req, state).await {
            Ok(value) => Ok(Bytes(value)),
            Err(rejection) => Err(Rejection::invalid_body(
                rejection.status(),
                "could not read request body",
            )),
        }
    }
}

/// Plain-text body extractor.
#[derive(Debug, Clone)]
pub struct Text(pub String);

impl<S> FromRequest<S> for Text
where
    S: Send + Sync,
{
    type Rejection = Rejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match String::from_request(req, state).await {
            Ok(value) => Ok(Text(value)),
            Err(rejection) => Err(Rejection::invalid_body(
                rejection.status(),
                "could not read request body",
            )),
        }
    }
}

/// Parse a required typed header from already-extracted [`Metadata`].
///
/// Called in-route by generated code (the header name and type are known there);
/// a name-less runtime extractor couldn't know them without generated marker
/// types. A missing or unparseable header yields a runtime-standard `400`.
///
/// # Errors
///
/// Returns a rejection when the header is absent, is not valid text, or cannot
/// be parsed as `T`.
pub fn required_header<T>(meta: &Metadata, name: &str) -> Result<T, Rejection>
where
    T: std::str::FromStr,
{
    let value = meta
        .headers()
        .get(name)
        .ok_or_else(|| Rejection::missing_header(format!("missing required header `{name}`")))?;
    parse_header_value(value, name)
}

/// Parse an optional typed header from already-extracted [`Metadata`].
///
/// # Errors
///
/// Returns a rejection when a present header is not valid text or cannot be
/// parsed as `T`.
pub fn optional_header<T>(meta: &Metadata, name: &str) -> Result<Option<T>, Rejection>
where
    T: std::str::FromStr,
{
    match meta.headers().get(name) {
        None => Ok(None),
        Some(value) => parse_header_value(value, name).map(Some),
    }
}

/// Parse a required typed cookie from already-extracted [`Metadata`].
///
/// # Errors
///
/// Returns a rejection when the cookie is absent, the cookie header is not
/// valid text, or the value cannot be parsed as `T`.
pub fn required_cookie<T>(meta: &Metadata, name: &str) -> Result<T, Rejection>
where
    T: std::str::FromStr,
{
    optional_cookie(meta, name)?
        .ok_or_else(|| Rejection::missing_cookie(format!("missing required cookie `{name}`")))
}

/// Parse an optional typed cookie from already-extracted [`Metadata`].
///
/// # Errors
///
/// Returns a rejection when the cookie header is not valid text or a present
/// value cannot be parsed as `T`.
pub fn optional_cookie<T>(meta: &Metadata, name: &str) -> Result<Option<T>, Rejection>
where
    T: std::str::FromStr,
{
    let Some(value) = cookie_value(meta, name)? else {
        return Ok(None);
    };
    value
        .parse::<T>()
        .map(Some)
        .map_err(|_| Rejection::invalid_cookie(format!("cookie `{name}` is malformed")))
}

fn parse_header_value<T>(value: &http::HeaderValue, name: &str) -> Result<T, Rejection>
where
    T: std::str::FromStr,
{
    let text = value
        .to_str()
        .map_err(|_| Rejection::invalid_header(format!("header `{name}` is not valid text")))?;
    text.parse::<T>()
        .map_err(|_| Rejection::invalid_header(format!("header `{name}` is malformed")))
}

fn cookie_value(meta: &Metadata, name: &str) -> Result<Option<String>, Rejection> {
    for value in meta.headers().get_all(http::header::COOKIE) {
        let text = value
            .to_str()
            .map_err(|_| Rejection::invalid_cookie("cookie header is not valid text"))?;
        for pair in text.split(';') {
            let Some((cookie_name, cookie_value)) = pair.trim().split_once('=') else {
                continue;
            };
            if cookie_name.trim() == name {
                return Ok(Some(cookie_value.trim().to_string()));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::extract::{FromRequest, FromRequestParts};
    use axum::routing::get;
    use tower::ServiceExt;

    fn meta_with(headers: HeaderMap) -> Metadata {
        Metadata {
            method: Method::GET,
            uri: Uri::from_static("/"),
            headers,
            extensions: Extensions::new(),
        }
    }

    // The repeated-key array encoding the generated client emits (`?x=1&x=2`).
    #[tokio::test]
    async fn query_handles_repeated_keys() {
        #[derive(serde::Deserialize)]
        struct Q {
            x: Vec<i32>,
        }
        let (mut parts, ()) = http::Request::builder()
            .uri("/items?x=1&x=2")
            .body(())
            .unwrap()
            .into_parts();
        let Query(q): Query<Q> = Query::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(q.x, vec![1, 2]);
    }

    #[tokio::test]
    async fn query_missing_optional_is_none() {
        #[derive(serde::Deserialize)]
        struct Q {
            x: Option<i32>,
        }
        let (mut parts, ()) = http::Request::builder()
            .uri("/items")
            .body(())
            .unwrap()
            .into_parts();
        let Query(q): Query<Q> = Query::from_request_parts(&mut parts, &()).await.unwrap();
        assert!(q.x.is_none());
    }

    #[tokio::test]
    async fn invalid_query_has_a_stable_kind() {
        #[derive(Debug, serde::Deserialize)]
        struct Q {
            #[serde(rename = "x")]
            _x: i32,
        }
        let (mut parts, ()) = http::Request::builder()
            .uri("/items?x=not-a-number")
            .body(())
            .unwrap()
            .into_parts();

        let rejection = Query::<Q>::from_request_parts(&mut parts, &())
            .await
            .unwrap_err();

        assert_eq!(rejection.kind(), RejectionKind::InvalidQuery);
        assert_eq!(rejection.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn invalid_path_has_a_stable_kind() {
        async fn handler(path: Result<Path<i32>, Rejection>) -> String {
            match path {
                Ok(_) => "ok".to_string(),
                Err(rejection) => format!("{:?}", rejection.kind()),
            }
        }

        let response = axum::Router::new()
            .route("/{id}", get(handler))
            .oneshot(
                http::Request::builder()
                    .uri("/not-a-number")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();

        assert_eq!(body.as_ref(), b"InvalidPath");
    }

    #[tokio::test]
    async fn json_distinguishes_media_type_from_body_failures() {
        #[derive(Debug, serde::Deserialize)]
        struct Payload {
            #[serde(rename = "value")]
            _value: i32,
        }

        let missing_media_type = http::Request::builder()
            .body(Body::from(r#"{"value":1}"#))
            .unwrap();
        let rejection = Json::<Payload>::from_request(missing_media_type, &())
            .await
            .unwrap_err();
        assert_eq!(rejection.kind(), RejectionKind::UnsupportedMediaType);
        assert_eq!(rejection.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

        let malformed = http::Request::builder()
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from("{"))
            .unwrap();
        let rejection = Json::<Payload>::from_request(malformed, &())
            .await
            .unwrap_err();
        assert_eq!(rejection.kind(), RejectionKind::InvalidBody);
        assert_eq!(rejection.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn form_decode_failure_has_a_stable_kind() {
        #[derive(Debug, serde::Deserialize)]
        struct Payload {
            #[serde(rename = "value")]
            _value: i32,
        }

        let request = http::Request::builder()
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("value=not-a-number"))
            .unwrap();
        let rejection = Form::<Payload>::from_request(request, &())
            .await
            .unwrap_err();

        assert_eq!(rejection.kind(), RejectionKind::InvalidBody);
    }

    fn failing_body() -> Body {
        Body::from_stream(futures::stream::once(async {
            Err::<bytes::Bytes, std::io::Error>(std::io::Error::other("read failed"))
        }))
    }

    #[tokio::test]
    async fn raw_body_read_failures_have_a_stable_kind() {
        let bytes_rejection = Bytes::from_request(http::Request::new(failing_body()), &())
            .await
            .unwrap_err();
        let text_rejection = Text::from_request(http::Request::new(failing_body()), &())
            .await
            .unwrap_err();

        assert_eq!(bytes_rejection.kind(), RejectionKind::InvalidBody);
        assert_eq!(text_rejection.kind(), RejectionKind::InvalidBody);
    }

    #[test]
    fn required_header_parses_present_and_rejects_absent() {
        let mut headers = HeaderMap::new();
        headers.insert("x-num", http::HeaderValue::from_static("42"));
        let meta = meta_with(headers);
        let n: i32 = required_header(&meta, "x-num").unwrap();
        assert_eq!(n, 42);
        let err = required_header::<i32>(&meta, "absent").unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert_eq!(err.kind(), RejectionKind::MissingHeader);
    }

    #[test]
    fn malformed_header_has_a_stable_kind() {
        let mut headers = HeaderMap::new();
        headers.insert("x-num", http::HeaderValue::from_static("not-a-number"));
        let meta = meta_with(headers);

        let err = required_header::<i32>(&meta, "x-num").unwrap_err();

        assert_eq!(err.kind(), RejectionKind::InvalidHeader);
    }

    #[test]
    fn optional_header_absent_is_none() {
        let meta = meta_with(HeaderMap::new());
        assert!(optional_header::<i32>(&meta, "absent").unwrap().is_none());
    }

    #[test]
    fn required_cookie_parses_present_and_rejects_absent() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::COOKIE,
            http::HeaderValue::from_static("session=42; theme=dark"),
        );
        let meta = meta_with(headers);

        let session: i32 = required_cookie(&meta, "session").unwrap();
        assert_eq!(session, 42);
        let theme: String = required_cookie(&meta, "theme").unwrap();
        assert_eq!(theme, "dark");
        let err = required_cookie::<i32>(&meta, "absent").unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert_eq!(err.kind(), RejectionKind::MissingCookie);
    }

    #[test]
    fn malformed_cookie_has_a_stable_kind() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::COOKIE,
            http::HeaderValue::from_static("session=not-a-number"),
        );
        let meta = meta_with(headers);

        let err = required_cookie::<i32>(&meta, "session").unwrap_err();

        assert_eq!(err.kind(), RejectionKind::InvalidCookie);
    }

    #[test]
    fn optional_cookie_absent_is_none() {
        let meta = meta_with(HeaderMap::new());
        assert!(optional_cookie::<i32>(&meta, "absent").unwrap().is_none());
    }

    // A required array query param the generator marks `#[serde(default)]` must
    // deserialize an absent key (an empty-array client request) to an empty Vec,
    // not a "missing field" error.
    #[tokio::test]
    async fn query_required_seq_defaults_when_absent() {
        #[derive(serde::Deserialize)]
        struct Q {
            #[serde(default)]
            tags: Vec<i32>,
        }
        let (mut parts, ()) = http::Request::builder()
            .uri("/items")
            .body(())
            .unwrap()
            .into_parts();
        let Query(q): Query<Q> = Query::from_request_parts(&mut parts, &()).await.unwrap();
        assert!(q.tags.is_empty());
    }

    #[test]
    fn internal_rejection_never_carries_the_error_value() {
        let rejection = Rejection::internal();

        assert_eq!(rejection.kind(), RejectionKind::Internal);
        assert_eq!(rejection.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(rejection.message(), "Internal Server Error");
    }
}
