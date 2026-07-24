// Copyright 2026 Oxide Computer Company

//! The server error model.

/// A boxed, thread-safe error used for infrastructure / unexpected failures.
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// The error half of a generated server trait method's return type.
///
/// This is a **data** type: it carries *what* went wrong, not *how* to encode
/// it. There is deliberately **no** blanket `IntoResponse` impl — a generic impl
/// cannot know whether `E = ()` means an empty body, whether `E` is raw vs JSON,
/// or which status a response variant implies. Encoding is done by the
/// generated per-operation responder, which has that per-operation knowledge
/// (see the crate docs and the generated `*_respond` helpers).
///
/// `E` represents every declared error response. It is a payload type when the
/// operation declares exactly one concrete error status and a generated enum
/// otherwise.
pub enum ServerError<E> {
    /// A declared, typed error response.
    Api(E),
    /// An infrastructure / unexpected failure. Encoded as `500 Internal Server
    /// Error` with a generic body (the underlying error is not leaked to the
    /// client).
    Internal(BoxError),
    /// A fully custom HTTP response — the deliberate escape hatch for anything
    /// the typed model can't express. Returned verbatim.
    Response(axum::response::Response),
}

impl<E> ServerError<E> {
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
            ServerError::Api(_) => f.write_str("Api(..)"),
            ServerError::Internal(e) => f.debug_tuple("Internal").field(e).finish(),
            ServerError::Response(_) => f.write_str("Response(..)"),
        }
    }
}
