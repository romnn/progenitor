// Copyright 2026 Oxide Computer Company

//! The service trait that bridges a generated server adapter to a router.

/// Implemented by every generated `{Api}Server<T>` adapter: it converts the
/// adapter (holding the user's `T: {Api}` implementation) into an
/// [`axum::Router`] with every route wired.
///
/// Always available (it needs only `axum`, which is non-optional) so power users
/// can `into_router()` and compose the result into their own server without the
/// `transport` feature.
pub trait Service {
    /// Consume the adapter, producing a fully-wired router.
    fn into_router(self) -> axum::Router;
}
