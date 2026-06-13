// Copyright 2026 Oxide Computer Company

//! The typed request wrapper handed to generated trait methods.

use crate::extract::Metadata;

/// A generated trait method's argument: the typed, decoded request `message`
/// plus the request [`Metadata`] (method, URI, headers, extensions).
///
/// The spec-declared, typed parameters (path / query / header) and the body are
/// bundled into `message` (a generated `{Op}Request` struct), so the implementor
/// never parses anything. The generic [`Metadata`] carries the *undeclared*
/// extras — raw headers, the full URI, request extensions (e.g. `ConnectInfo`,
/// middleware-inserted auth) — for cross-cutting needs. This is the REST analog
/// of tonic's `Request<T>` (message + metadata).
#[derive(Debug, Clone)]
pub struct Request<T> {
    message: T,
    meta: Metadata,
}

impl<T> Request<T> {
    /// Create a request with empty metadata. Useful for tests/fixtures.
    pub fn new(message: T) -> Self {
        Request {
            message,
            meta: Metadata::default(),
        }
    }

    /// Create a request from extracted [`Metadata`] and a decoded message.
    pub fn from_metadata(meta: Metadata, message: T) -> Self {
        Request { message, meta }
    }

    /// Borrow the decoded message.
    pub fn get_ref(&self) -> &T {
        &self.message
    }

    /// Mutably borrow the decoded message.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.message
    }

    /// Consume the request, returning the decoded message.
    pub fn into_inner(self) -> T {
        self.message
    }

    /// The request metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    /// The request headers.
    pub fn headers(&self) -> &http::HeaderMap {
        self.meta.headers()
    }

    /// The request extensions (`ConnectInfo`, auth results, ...).
    pub fn extensions(&self) -> &http::Extensions {
        self.meta.extensions()
    }

    /// The request method.
    pub fn method(&self) -> &http::Method {
        self.meta.method()
    }

    /// The request URI.
    pub fn uri(&self) -> &http::Uri {
        self.meta.uri()
    }

    /// Consume the request, returning its metadata and message.
    pub fn into_parts(self) -> (Metadata, T) {
        (self.meta, self.message)
    }
}
