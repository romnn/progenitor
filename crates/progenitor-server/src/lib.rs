// Copyright 2026 Oxide Computer Company

//! Server-side runtime support for progenitor-generated servers.
//!
//! This is the runtime counterpart to the code progenitor emits when server
//! generation is enabled. It is analogous to `progenitor-client` for the client
//! side, and is deliberately **never** re-exported by the build.rs-facing
//! generator crates: a consumer that wants a server adds `progenitor-server` to
//! its own `[dependencies]` (exactly like the `tonic-build`/`tonic` split).
//!
//! Generated code references this crate as `progenitor_server::*` and pulls the
//! support surface in via `use progenitor_server::codegen::*;`.
//!
//! # Shape
//!
//! For each operation the generator emits a trait method
//!
//! ```ignore
//! async fn op(&self, request: Request<OpRequest>) -> Result<Response<Success>, OpError>;
//! ```
//!
//! where `OpRequest` bundles the typed path/query/header params and body, and
//! `OpError` is `ServerError<E>` for the operation's declared error type `E`.
//! A generated `{Api}Server<T>` adapter turns any `T: {Api}` into an
//! [`axum::Router`], doing all (de)serialization so the implementor writes only
//! business logic.

#![deny(missing_docs)]

mod error;
mod extract;
pub mod multipart;
mod request;
mod response;
mod service;
mod status;

pub use error::{BoxError, ServerError};
pub use extract::{
    Bytes, Form, Json, Metadata, Multipart, Path, Query, Rejection, RejectionKind, Text,
    optional_cookie, optional_header, required_cookie, required_header,
};
pub use request::Request;
pub use response::{Response, respond};
pub use service::Service;
pub use status::ClassStatus;

#[cfg(feature = "transport")]
mod transport;
#[cfg(feature = "transport")]
pub use transport::{Bound, Router, Server};

/// Support surface imported by generated server code via
/// `use progenitor_server::codegen::*;`.
///
/// This mirrors tonic's `tonic::codegen` trick: generated code names `axum`,
/// `async_trait`, `bytes`, etc. **only** through these re-exports, so a crate
/// that contains generated server code needs `progenitor-server` as its single
/// new direct dependency — axum/serde/async-trait arrive transitively.
pub mod codegen {
    pub use async_trait::async_trait;
    pub use axum;
    pub use bytes;
    pub use http;
    pub use serde;
    pub use std::sync::Arc;
}
