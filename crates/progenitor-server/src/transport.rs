// Copyright 2026 Oxide Computer Company

//! The tonic-style serving convenience: build a server, register one or more
//! services, and serve. Gated behind the `transport` feature (the only part that
//! pulls `tokio` + the axum server). Power users who want full control can
//! instead call [`Service::into_router`](crate::Service::into_router) and serve
//! the [`axum::Router`] themselves.

use std::future::Future;
use std::net::SocketAddr;

use crate::service::Service;

/// Builder for a server. Register services with [`Server::add_service`], which
/// returns a [`Router`] you can chain further services onto and then serve.
///
/// ```ignore
/// progenitor_server::Server::builder()
///     .add_service(my_api::server::PetstoreServer::new(impl_value))
///     .serve("0.0.0.0:8080".parse()?)
///     .await?;
/// ```
#[derive(Default)]
pub struct Server {
    router: axum::Router,
}

impl Server {
    /// Start building a server.
    pub fn builder() -> Self {
        Server {
            router: axum::Router::new(),
        }
    }

    /// Register the first service, returning a [`Router`].
    pub fn add_service<S: Service>(self, service: S) -> Router {
        Router {
            router: self.router.merge(service.into_router()),
        }
    }
}

/// One or more registered services, ready to serve. Multiple services (multiple
/// specs, or per-tag traits) can be mounted on one socket.
pub struct Router {
    router: axum::Router,
}

impl Router {
    /// Register an additional service.
    pub fn add_service<S: Service>(mut self, service: S) -> Self {
        self.router = self.router.merge(service.into_router());
        self
    }

    /// Escape hatch: take the underlying [`axum::Router`] to compose with your
    /// own middleware / server.
    ///
    /// To add a tower [`Layer`](tower::Layer) (timeouts, concurrency limits, a
    /// larger request-body limit via `axum::extract::DefaultBodyLimit`, etc.),
    /// apply it here: `router.into_router().layer(my_layer)`. The v1 builder
    /// intentionally does not re-expose axum's full layer API; the default
    /// request-body limit is axum's (2 MiB) unless raised this way.
    pub fn into_router(self) -> axum::Router {
        self.router
    }

    /// The router as a `MakeService`, for `axum::serve(listener,
    /// router.into_make_service())` or for `into_make_service_with_connect_info`
    /// on the underlying router.
    pub fn into_make_service(self) -> axum::routing::IntoMakeService<axum::Router> {
        self.router.into_make_service()
    }

    /// Bind `addr` and return a [`Bound`] server. Bind-first so callers (and
    /// tests) can read the actually-bound port via [`Bound::local_addr`] before
    /// serving — essential for ephemeral `127.0.0.1:0` binds.
    pub async fn bind(self, addr: SocketAddr) -> std::io::Result<Bound> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        Bound::new(listener, self.router)
    }

    /// Bind `addr` and serve until the process ends.
    pub async fn serve(self, addr: SocketAddr) -> std::io::Result<()> {
        self.bind(addr).await?.serve().await
    }

    /// Serve on an already-bound listener.
    pub async fn serve_listener(self, listener: tokio::net::TcpListener) -> std::io::Result<()> {
        Bound::new(listener, self.router)?.serve().await
    }
}

/// A bound server: a listener plus the router. Read [`local_addr`](Bound::local_addr)
/// then [`serve`](Bound::serve).
pub struct Bound {
    listener: tokio::net::TcpListener,
    local_addr: SocketAddr,
    router: axum::Router,
}

impl Bound {
    fn new(listener: tokio::net::TcpListener, router: axum::Router) -> std::io::Result<Self> {
        let local_addr = listener.local_addr()?;
        Ok(Bound {
            listener,
            local_addr,
            router,
        })
    }

    /// The actually-bound local address (resolves `:0` to the chosen port).
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Serve until the process ends.
    pub async fn serve(self) -> std::io::Result<()> {
        axum::serve(self.listener, self.router).await
    }

    /// Serve until `signal` resolves, then shut down gracefully.
    pub async fn serve_with_shutdown<F>(self, signal: F) -> std::io::Result<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(signal)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Service;

    struct Svc(&'static str);

    impl Service for Svc {
        fn into_router(self) -> axum::Router {
            axum::Router::new().route(self.0, axum::routing::get(|| async { "ok" }))
        }
    }

    #[tokio::test]
    async fn bind_reports_ephemeral_port() {
        let bound = Server::builder()
            .add_service(Svc("/a"))
            .bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        assert_ne!(bound.local_addr().port(), 0);
    }

    // Acceptance: two services mount on one Server.
    #[tokio::test]
    async fn multi_service_mounts_all() {
        use tower::ServiceExt;

        let router = Server::builder()
            .add_service(Svc("/a"))
            .add_service(Svc("/b"))
            .into_router();

        for path in ["/a", "/b"] {
            let response = router
                .clone()
                .oneshot(
                    http::Request::builder()
                        .uri(path)
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), http::StatusCode::OK);
        }
    }
}
