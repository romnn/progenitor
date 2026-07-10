// Copyright 2026 Oxide Computer Company

//! The server error model.

use http::StatusCode;

/// A boxed, thread-safe error used for infrastructure / unexpected failures.
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// The error half of a generated server trait method's return type.
///
/// This is a **data** type: it carries *what* went wrong, not *how* to encode
/// it. There is deliberately **no** blanket `IntoResponse` impl — a generic impl
/// cannot know whether `E = ()` means an empty body, whether `E` is raw vs JSON,
/// or whether an `Api { status, body }` carries a status matching the operation
/// contract. Encoding is done by the generated per-operation responder, which
/// has that per-op knowledge (see the crate docs and the generated `*_respond`
/// helpers).
///
/// `E` is the operation's declared error payload type (often a typed struct, but
/// possibly `()` for a bodyless error or `bytes::Bytes` for a raw one).
pub enum ServerError<E> {
    /// A declared, typed error response: encoded with `status` and the body
    /// rendered per the operation's response kind (typed → JSON, `()` → empty,
    /// `bytes::Bytes` → raw).
    Api {
        /// The HTTP status to return.
        status: StatusCode,
        /// The typed error payload.
        body: E,
    },
    /// An infrastructure / unexpected failure. Encoded as `500 Internal Server
    /// Error` with a generic body (the underlying error is not leaked to the
    /// client).
    Internal(BoxError),
    /// A fully custom HTTP response — the deliberate escape hatch for anything
    /// the typed model can't express. Returned verbatim.
    Response(axum::response::Response),
}

impl<E> ServerError<E> {
    /// Construct a typed [`ServerError::Api`] error.
    pub fn api(status: StatusCode, body: E) -> Self {
        ServerError::Api { status, body }
    }

    /// Construct a [`ServerError::Internal`] (`500`) from any error.
    pub fn internal(error: impl Into<BoxError>) -> Self {
        ServerError::Internal(error.into())
    }

    /// Construct a [`ServerError::Response`] escape hatch from a raw response.
    #[must_use]
    pub fn response(response: axum::response::Response) -> Self {
        ServerError::Response(response)
    }
}

impl<E> std::fmt::Debug for ServerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::Api { status, .. } => f
                .debug_struct("Api")
                .field("status", status)
                .finish_non_exhaustive(),
            ServerError::Internal(e) => f.debug_tuple("Internal").field(e).finish(),
            ServerError::Response(_) => f.write_str("Response(..)"),
        }
    }
}
