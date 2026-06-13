// Copyright 2026 Oxide Computer Company

//! Request extractors with controlled, runtime-standard rejections.
//!
//! Generated routes use these custom extractors instead of the bare axum ones so
//! that an extraction failure produces *our* standardized HTTP error (a uniform
//! `400`/`415`) rather than axum's default rejection body — and so the query
//! extractor handles the repeated-key array encoding the generated client emits
//! (`?x=1&x=2`), which axum's `serde_urlencoded`-based `Query` cannot.
//!
//! Extraction failures are intentionally **runtime-standard**, not the
//! operation's typed error: a generic extractor can't know an operation's
//! declared error type. A generated client may therefore surface such a `400` as
//! an undecodable payload; the real status is observable via raw HTTP.

use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::response::{IntoResponse, Response};
use http::request::Parts;
use http::{Extensions, HeaderMap, Method, StatusCode, Uri};
use serde::de::DeserializeOwned;

/// A standardized extraction rejection: an HTTP status plus a short message.
///
/// Its [`IntoResponse`] is what an extractor short-circuits with, so every
/// generated route emits a uniform error shape for malformed requests.
#[derive(Debug, Clone)]
pub struct Rejection {
    status: StatusCode,
    message: String,
}

impl Rejection {
    /// Construct a rejection with an explicit status.
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Rejection {
            status,
            message: message.into(),
        }
    }

    /// A `400 Bad Request` rejection.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Rejection::new(StatusCode::BAD_REQUEST, message)
    }

    /// A `415 Unsupported Media Type` rejection.
    pub fn unsupported_media_type(message: impl Into<String>) -> Self {
        Rejection::new(StatusCode::UNSUPPORTED_MEDIA_TYPE, message)
    }

    /// The HTTP status this rejection encodes to.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// The human-readable message.
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
            Err(rejection) => Err(Rejection::new(rejection.status(), "invalid path parameter")),
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
            Err(error) => Err(Rejection::bad_request(format!(
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
            Err(rejection) => Err(Rejection::new(rejection.status(), "invalid JSON body")),
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
            Err(rejection) => Err(Rejection::new(rejection.status(), "invalid form body")),
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
            Err(rejection) => Err(Rejection::new(
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
            Err(rejection) => Err(Rejection::new(
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
pub fn required_header<T>(meta: &Metadata, name: &str) -> Result<T, Rejection>
where
    T: std::str::FromStr,
{
    let value = meta
        .headers()
        .get(name)
        .ok_or_else(|| Rejection::bad_request(format!("missing required header `{name}`")))?;
    parse_header_value(value, name)
}

/// Parse an optional typed header from already-extracted [`Metadata`].
pub fn optional_header<T>(meta: &Metadata, name: &str) -> Result<Option<T>, Rejection>
where
    T: std::str::FromStr,
{
    match meta.headers().get(name) {
        None => Ok(None),
        Some(value) => parse_header_value(value, name).map(Some),
    }
}

fn parse_header_value<T>(value: &http::HeaderValue, name: &str) -> Result<T, Rejection>
where
    T: std::str::FromStr,
{
    let text = value
        .to_str()
        .map_err(|_| Rejection::bad_request(format!("header `{name}` is not valid text")))?;
    text.parse::<T>()
        .map_err(|_| Rejection::bad_request(format!("header `{name}` is malformed")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::FromRequestParts;

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
        let (mut parts, _) = http::Request::builder()
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
        let (mut parts, _) = http::Request::builder()
            .uri("/items")
            .body(())
            .unwrap()
            .into_parts();
        let Query(q): Query<Q> = Query::from_request_parts(&mut parts, &()).await.unwrap();
        assert!(q.x.is_none());
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
    }

    #[test]
    fn optional_header_absent_is_none() {
        let meta = meta_with(HeaderMap::new());
        assert!(optional_header::<i32>(&meta, "absent").unwrap().is_none());
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
        let (mut parts, _) = http::Request::builder()
            .uri("/items")
            .body(())
            .unwrap()
            .into_parts();
        let Query(q): Query<Q> = Query::from_request_parts(&mut parts, &()).await.unwrap();
        assert!(q.tags.is_empty());
    }
}
