// Copyright 2026 Oxide Computer Company

//! The typed response wrapper returned by generated trait methods, and the
//! low-level response encoders the generated per-operation responders call.

use http::HeaderMap;

/// The success half of a generated server trait method's return type: a typed
/// `message` plus extra headers.
///
/// The message type determines the HTTP status. It is the response payload when
/// an operation declares exactly one concrete success status and a generated
/// enum otherwise.
#[derive(Debug, Clone)]
#[must_use = "a response must be returned or converted into an HTTP response"]
pub struct Response<T> {
    message: T,
    headers: HeaderMap,
}

impl<T> Response<T> {
    /// Wrap a message with no extra headers.
    pub fn new(message: T) -> Self {
        Response {
            message,
            headers: HeaderMap::new(),
        }
    }

    /// Insert an extra response header.
    pub fn with_header(mut self, key: http::HeaderName, value: http::HeaderValue) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Borrow the message.
    pub fn get_ref(&self) -> &T {
        &self.message
    }

    /// Mutably borrow the message.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.message
    }

    /// The extra response headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Mutably borrow the extra response headers.
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Consume the response, returning the message.
    pub fn into_inner(self) -> T {
        self.message
    }

    /// Consume the response, returning `(extra headers, message)`.
    pub fn into_parts(self) -> (HeaderMap, T) {
        (self.headers, self.message)
    }
}

/// Low-level response encoders used by the generated per-operation responders.
///
/// The generator selects the encoder by the operation's response kind (typed →
/// [`respond::json`], `None` → [`respond::empty`], raw →
/// [`respond::bytes`]), derives the status from the response type, and merges
/// any [`Response`] headers before calling one of these. Keeping the encoders
/// here (not in generated code) makes them unit-testable and lets the wire
/// format evolve without regenerating.
pub mod respond {
    use super::HeaderMap;
    use axum::response::{IntoResponse, Response};
    use http::{StatusCode, header};

    fn merge_headers(dst: &mut HeaderMap, src: HeaderMap) {
        // `HeaderMap::into_iter` yields the key only on the first value of a
        // repeated header and `None` for each subsequent value. Carry the last
        // seen key forward and `append` (not `insert`) so multi-valued headers
        // like `Set-Cookie` survive the merge.
        let mut last_key: Option<http::HeaderName> = None;
        for (key, value) in src {
            let key = match key {
                Some(key) => {
                    last_key = Some(key.clone());
                    key
                }
                None => match &last_key {
                    Some(key) => key.clone(),
                    None => continue,
                },
            };
            dst.append(key, value);
        }
    }

    /// Encode a JSON body with `status` and merged `headers`.
    pub fn json<T: serde::Serialize>(status: StatusCode, headers: HeaderMap, body: &T) -> Response {
        match serde_json::to_vec(body) {
            Ok(buf) => {
                let mut response = (status, buf).into_response();
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/json"),
                );
                merge_headers(response.headers_mut(), headers);
                response
            }
            // A success/typed-error body that won't serialize is a server bug.
            Err(error) => internal(Box::new(error)),
        }
    }

    /// Encode an empty body (status only) with merged `headers`.
    #[must_use]
    pub fn empty(status: StatusCode, headers: HeaderMap) -> Response {
        let mut response = status.into_response();
        merge_headers(response.headers_mut(), headers);
        response
    }

    /// Encode a raw byte body with an explicit `content_type` and merged
    /// `headers`.
    pub fn bytes(
        status: StatusCode,
        headers: HeaderMap,
        content_type: &str,
        body: bytes::Bytes,
    ) -> Response {
        let mut response = (status, body).into_response();
        if let Ok(value) = header::HeaderValue::from_str(content_type) {
            response.headers_mut().insert(header::CONTENT_TYPE, value);
        }
        merge_headers(response.headers_mut(), headers);
        response
    }

    /// Encode a `500 Internal Server Error` with a generic body. The underlying
    /// error is intentionally not leaked to the client.
    #[must_use]
    pub fn internal(_error: crate::BoxError) -> Response {
        // TODO: surface `_error` through `tracing` once that dependency lands.
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{
        StatusCode,
        header::{CONTENT_TYPE, SET_COOKIE},
    };

    #[test]
    fn json_sets_content_type_and_status() {
        let response = respond::json(
            StatusCode::OK,
            HeaderMap::new(),
            &serde_json::json!({"a": 1}),
        );
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[test]
    fn empty_is_status_only() {
        let response = respond::empty(StatusCode::NO_CONTENT, HeaderMap::new());
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn bytes_sets_given_content_type() {
        let response = respond::bytes(
            StatusCode::OK,
            HeaderMap::new(),
            "application/octet-stream",
            bytes::Bytes::from_static(b"x"),
        );
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/octet-stream"
        );
    }

    // merge_headers must preserve repeated (multi-valued) response headers.
    #[test]
    fn merge_preserves_repeated_headers() {
        let mut extra = HeaderMap::new();
        extra.append(SET_COOKIE, http::HeaderValue::from_static("a=1"));
        extra.append(SET_COOKIE, http::HeaderValue::from_static("b=2"));
        let response = respond::json(StatusCode::OK, extra, &serde_json::json!({}));
        assert_eq!(response.headers().get_all(SET_COOKIE).iter().count(), 2);
    }
}
