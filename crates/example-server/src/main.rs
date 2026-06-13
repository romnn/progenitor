// Copyright 2026 Oxide Computer Company

//! Example of progenitor's opt-in server generation.
//!
//! `generate_api!(server = true, ...)` emits, alongside the usual `Client`, a
//! `server` module containing an `Example` service trait and an `ExampleServer`
//! adapter. Implement the trait and mount the adapter — the generated server is
//! wire-compatible with the generated `Client`.

use progenitor::generate_api;

generate_api!(
    spec = "../../sample_openapi/example-server.json",
    server = true,
);

use progenitor_server::codegen::async_trait;
use progenitor_server::codegen::bytes::Bytes;
use progenitor_server::{Request, Response, ServerError};

/// The user's implementation — just business logic; no (de)serialization.
#[derive(Default)]
struct Example;

#[async_trait]
impl server::Example for Example {
    async fn ping(
        &self,
        _request: Request<server::PingRequest>,
    ) -> Result<Response<types::Message>, server::PingError> {
        Ok(Response::new(types::Message {
            message: "pong".to_string(),
        }))
    }

    async fn echo(
        &self,
        request: Request<server::EchoRequest>,
    ) -> Result<Response<types::Message>, server::EchoError> {
        let req = request.get_ref();
        if req.text.is_empty() {
            return Err(ServerError::api(
                progenitor_server::codegen::http::StatusCode::BAD_REQUEST,
                types::Error {
                    code: 400,
                    message: "text must not be empty".to_string(),
                },
            ));
        }
        let message = match &req.suffix {
            Some(suffix) => format!("{}{}", req.text, suffix),
            None => req.text.clone(),
        };
        Ok(Response::new(types::Message { message }))
    }

    async fn maybe(
        &self,
        request: Request<server::MaybeRequest>,
    ) -> Result<Response<server::MaybeResponse>, server::MaybeError> {
        match request.get_ref().mode.as_str() {
            "bytes" => Ok(Response::new(server::MaybeResponse::Status206(
                Bytes::from_static(b"partial"),
            ))),
            "busy" => Err(server::MaybeErrorResponse::Default {
                status: progenitor_server::codegen::http::StatusCode::SERVICE_UNAVAILABLE,
                body: Bytes::from_static(b"busy"),
            }
            .into()),
            "auth" => Err(server::MaybeErrorResponse::Status401(types::Error {
                code: 401,
                message: "authentication required".to_string(),
            })
            .into()),
            "bad" => Err(server::MaybeErrorResponse::StatusRange4xx {
                status: progenitor_server::codegen::http::StatusCode::BAD_REQUEST,
                body: types::Error {
                    code: 400,
                    message: "bad mode".to_string(),
                },
            }
            .into()),
            mode => Ok(Response::new(server::MaybeResponse::Status200(
                types::Message {
                    message: mode.to_string(),
                },
            ))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Power-user path: compose the router into your own axum app.
    let _router = server::ExampleServer::new(Example).into_router();

    // Convenience path (commented out so the example doesn't block):
    // progenitor_server::Server::builder()
    //     .add_service(server::ExampleServer::new(Example))
    //     .serve("127.0.0.1:8080".parse()?)
    //     .await?;

    Ok(())
}
