//! Generated Petstore 3.1 API client — conformance crate.
//!
//! Hermetic crate: spec is committed in this directory as petstore-31.yaml.
//! Used as a fast always-asserted PR-CI gate; the tiny spec ensures a quick
//! cold build and any regression in 3.1 baseline handling surfaces immediately.
//!
//! An exact response variant has no independent status slot, so a mismatched
//! status and payload cannot be constructed:
//!
//! ```compile_fail
//! use conformance_petstore_31::{server, types};
//! use progenitor_server::codegen::http::StatusCode;
//!
//! let body = types::Error {
//!     code: 418,
//!     message: "teapot".to_string(),
//! };
//! let declared = server::TypedErrorsErrorResponse::Status404(body);
//! let _ = server::TypedErrorsError::api(StatusCode::IM_A_TEAPOT, declared);
//! ```

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let _client = Client::new("https://petstore.example.com/v1");
    }

    #[test]
    fn pet_round_trips_through_serde() {
        let pet: types::Pet = serde_json::from_str(r#"{"id": 7, "name": "rex", "tag": "dog"}"#)
            .expect("deserializes");
        assert_eq!(pet.id, 7);
        assert_eq!(pet.name, "rex");
        assert_eq!(pet.tag.as_deref(), Some("dog"));

        let value = serde_json::to_value(&pet).expect("serializes");
        assert_eq!(value["name"], "rex");
        assert_eq!(value["id"], 7);
        assert_eq!(value["tag"], "dog");
    }

    #[test]
    fn optional_fields_may_be_absent() {
        let pet: types::Pet =
            serde_json::from_str(r#"{"id": 1, "name": "min"}"#).expect("deserializes");
        assert_eq!(pet.tag, None);

        let value = serde_json::to_value(&pet).expect("serializes");
        assert!(
            value.get("tag").is_none(),
            "absent optional must stay off wire"
        );
    }

    #[test]
    fn error_round_trips() {
        let error: types::Error =
            serde_json::from_str(r#"{"code": 404, "message": "not found"}"#).expect("deserializes");
        assert_eq!(error.code, 404);
        assert_eq!(error.message, "not found");
    }
}

#[cfg(test)]
mod example_tests {
    include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
}

#[cfg(test)]
pub mod mock {
    include!(concat!(env!("OUT_DIR"), "/mock.rs"));
}

pub mod server {
    include!(concat!(env!("OUT_DIR"), "/server.rs"));
}

#[cfg(test)]
mod operation_tests {
    use super::*;
    use conformance_support::httpmock::MockServer;

    #[conformance_support::tokio::test]
    async fn list_pets_sends_get_to_pets() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/pets");
                then.status(200).json_body(serde_json::json!([]));
            })
            .await;
        let client = Client::new(&server.base_url());
        let result = client.list_pets(None).await;
        mock.assert_async().await;
        assert!(result.is_ok(), "expected 200 success from mock");
    }

    #[conformance_support::tokio::test]
    async fn show_pet_by_id_constructs_correct_path() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/pets/42");
                then.status(200)
                    .json_body(serde_json::json!({"id": 42, "name": "Fido"}));
            })
            .await;
        let client = Client::new(&server.base_url());
        let _ = client.show_pet_by_id("42").await;
        mock.assert_async().await;
    }
}

// The headline server test: implement the generated `Petstore` trait, serve it
// on an ephemeral port, and drive it with the *generated client* — proving the
// generated server and client are wire-compatible end to end.
#[cfg(test)]
mod server_round_trip {
    use super::*;
    use progenitor_server::codegen::async_trait;
    use progenitor_server::codegen::http::StatusCode;
    use progenitor_server::{
        Rejection, RejectionKind, Request, Response, ServerError,
    };
    use std::sync::Mutex;

    #[derive(Default)]
    struct MyPets {
        store: Mutex<Vec<types::Pet>>,
    }

    #[async_trait]
    impl server::Petstore for MyPets {
        async fn list_pets(
            &self,
            _request: Request<server::ListPetsRequest>,
        ) -> Result<Response<types::Pets>, server::ListPetsError> {
            Ok(Response::new(types::Pets(
                self.store.lock().unwrap().clone(),
            )))
        }

        async fn create_pets(
            &self,
            _request: Request<server::CreatePetsRequest>,
        ) -> Result<Response<()>, server::CreatePetsError> {
            // createPets has no request body in petstore-31.yaml; seed one pet.
            self.store.lock().unwrap().push(types::Pet {
                id: 1,
                name: "Fido".to_string(),
                tag: None,
            });
            Ok(Response::new(()))
        }

        async fn show_pet_by_id(
            &self,
            request: Request<server::ShowPetByIdRequest>,
        ) -> Result<Response<types::Pet>, server::ShowPetByIdError> {
            let id: i64 = request.get_ref().pet_id.parse().unwrap_or(-1);
            let found = self
                .store
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .cloned();
            match found {
                Some(pet) => Ok(Response::new(pet)),
                None => Err(server::ShowPetByIdErrorResponse::Default {
                    status: StatusCode::NOT_FOUND,
                    body: types::Error {
                        code: 404,
                        message: "not found".to_string(),
                    },
                }
                .into()),
            }
        }

        async fn typed_errors(
            &self,
            request: Request<server::TypedErrorsRequest>,
        ) -> Result<Response<()>, server::TypedErrorsError> {
            let body = types::Error {
                code: 1,
                message: request.get_ref().kind.clone(),
            };
            match request.get_ref().kind.as_str() {
                "missing" => Err(server::TypedErrorsErrorResponse::Status404(body).into()),
                _ => Err(server::TypedErrorsErrorResponse::Status409(body).into()),
            }
        }

        async fn rejection_probe(
            &self,
            request: Request<server::RejectionProbeRequest>,
        ) -> Result<Response<()>, server::RejectionProbeError> {
            if request.get_ref().x_mode == "internal" {
                return Err(ServerError::internal(std::io::Error::other(
                    "private internal detail",
                )));
            }
            Ok(Response::new(()))
        }
    }

    struct CustomRejections(MyPets);

    #[derive(serde::Serialize)]
    struct RejectionEnvelope {
        code: &'static str,
        message: String,
    }

    #[async_trait]
    impl server::Petstore for CustomRejections {
        fn render_rejection(
            &self,
            rejection: Rejection,
        ) -> progenitor_server::codegen::axum::response::Response {
            let code = match rejection.kind() {
                RejectionKind::InvalidBody => "invalid_body",
                RejectionKind::UnsupportedMediaType => "unsupported_media_type",
                RejectionKind::InvalidQuery => "invalid_query",
                RejectionKind::InvalidPath => "invalid_path",
                RejectionKind::MissingHeader => "missing_header",
                RejectionKind::InvalidHeader => "invalid_header",
                RejectionKind::MissingCookie => "missing_cookie",
                RejectionKind::InvalidCookie => "invalid_cookie",
                RejectionKind::MissingPart => "missing_part",
                RejectionKind::InvalidPart => "invalid_part",
                RejectionKind::Internal => "internal",
                _ => "request_rejected",
            };
            let body = RejectionEnvelope {
                code,
                message: rejection.message().to_string(),
            };
            // The adapter owns status semantics, so this deliberately incorrect
            // status verifies that a renderer can only customize body and headers.
            progenitor_server::respond::json(
                StatusCode::IM_A_TEAPOT,
                progenitor_server::codegen::http::HeaderMap::new(),
                &body,
            )
        }

        async fn list_pets(
            &self,
            request: Request<server::ListPetsRequest>,
        ) -> Result<Response<types::Pets>, server::ListPetsError> {
            <MyPets as server::Petstore>::list_pets(&self.0, request).await
        }

        async fn create_pets(
            &self,
            request: Request<server::CreatePetsRequest>,
        ) -> Result<Response<()>, server::CreatePetsError> {
            <MyPets as server::Petstore>::create_pets(&self.0, request).await
        }

        async fn show_pet_by_id(
            &self,
            request: Request<server::ShowPetByIdRequest>,
        ) -> Result<Response<types::Pet>, server::ShowPetByIdError> {
            <MyPets as server::Petstore>::show_pet_by_id(&self.0, request).await
        }

        async fn typed_errors(
            &self,
            request: Request<server::TypedErrorsRequest>,
        ) -> Result<Response<()>, server::TypedErrorsError> {
            <MyPets as server::Petstore>::typed_errors(&self.0, request).await
        }

        async fn rejection_probe(
            &self,
            request: Request<server::RejectionProbeRequest>,
        ) -> Result<Response<()>, server::RejectionProbeError> {
            <MyPets as server::Petstore>::rejection_probe(&self.0, request).await
        }
    }

    async fn spawn_service<T: server::Petstore>(service: T) -> std::net::SocketAddr {
        let bound = progenitor_server::Server::builder()
            .add_service(server::PetstoreServer::new(service))
            .bind("127.0.0.1:0".parse().unwrap())
            .await
            .expect("bind ephemeral port");
        let addr = bound.local_addr();
        conformance_support::tokio::spawn(async move {
            bound.serve().await.unwrap();
        });
        addr
    }

    async fn spawn() -> std::net::SocketAddr {
        spawn_service(MyPets::default()).await
    }

    #[conformance_support::tokio::test]
    async fn create_then_show_round_trips_through_generated_client() {
        let addr = spawn().await;
        let client = Client::new(&format!("http://{addr}"));

        client.create_pets().await.expect("create 201");

        let pet = client
            .show_pet_by_id("1")
            .await
            .expect("show 200")
            .into_inner();
        assert_eq!(pet.id, 1);
        assert_eq!(pet.name, "Fido");

        // `Pets` is a transparent newtype that derefs to `Vec<Pet>`.
        let all = client.list_pets(None).await.expect("list 200").into_inner();
        assert_eq!(all.len(), 1);
    }

    #[conformance_support::tokio::test]
    async fn missing_pet_decodes_as_typed_error() {
        let addr = spawn().await;
        let client = Client::new(&format!("http://{addr}"));
        match client.show_pet_by_id("999").await {
            Err(Error::ErrorResponse(resp)) => {
                assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);
                assert_eq!(resp.into_inner().code, 404);
            }
            other => panic!("expected ErrorResponse, got {other:?}"),
        }
    }

    // A bare HTTP probe sees the real status, even where the typed client can't
    // (an empty error/501 body surfaces as InvalidResponsePayload — D-501).
    #[conformance_support::tokio::test]
    async fn raw_http_status_is_observable() {
        let addr = spawn().await;
        let response = reqwest::Client::new()
            .get(format!("http://{addr}/pets/999"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    }

    #[conformance_support::tokio::test]
    async fn same_payload_errors_preserve_their_declared_status() {
        let addr = spawn().await;
        let client = Client::new(&format!("http://{addr}"));

        let missing = client
            .typed_errors("missing")
            .await
            .expect_err("missing must return an error");
        match missing {
            Error::ErrorResponse(response) => {
                assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
                assert_eq!(response.into_inner().message, "missing");
            }
            other => panic!("expected typed 404 response, got {other:?}"),
        }

        let conflict = client
            .typed_errors("conflict")
            .await
            .expect_err("conflict must return an error");
        match conflict {
            Error::ErrorResponse(response) => {
                assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
                assert_eq!(response.into_inner().message, "conflict");
            }
            other => panic!("expected typed 409 response, got {other:?}"),
        }
    }

    #[conformance_support::tokio::test]
    async fn default_rejection_renderer_preserves_plain_text_body() {
        let addr = spawn().await;
        let response = reqwest::Client::new()
            .post(format!("http://{addr}/rejection-probe/1?limit=1"))
            .header("x-mode", "ok")
            .header(reqwest::header::COOKIE, "session=1")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body("{")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
        assert_eq!(response.text().await.unwrap(), "invalid JSON body");
    }

    fn rejection_probe_request(
        client: &reqwest::Client,
        url: &str,
        mode: &str,
    ) -> reqwest::RequestBuilder {
        client
            .post(url)
            .header("x-mode", mode)
            .header(reqwest::header::COOKIE, "session=1")
    }

    async fn rejection_body(
        response: reqwest::Response,
        status: reqwest::StatusCode,
        code: &str,
    ) -> serde_json::Value {
        assert_eq!(response.status(), status);
        let body = response.json::<serde_json::Value>().await.unwrap();
        assert_eq!(body["code"], code);
        body
    }

    #[conformance_support::tokio::test]
    async fn custom_renderer_controls_request_and_internal_failures() {
        let addr = spawn_service(CustomRejections(MyPets::default())).await;
        let client = reqwest::Client::new();
        let url = format!("http://{addr}/rejection-probe/1?limit=1");

        let invalid_body = rejection_probe_request(&client, &url, "ok")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body("{")
            .send()
            .await
            .unwrap();
        let body =
            rejection_body(invalid_body, reqwest::StatusCode::BAD_REQUEST, "invalid_body").await;
        assert_eq!(body["message"], "invalid JSON body");

        let missing_header = client
            .post(&url)
            .header(reqwest::header::COOKIE, "session=1")
            .json(&serde_json::json!({"id": 1, "name": "Fido"}))
            .send()
            .await
            .unwrap();
        let body = rejection_body(
            missing_header,
            reqwest::StatusCode::BAD_REQUEST,
            "missing_header",
        )
        .await;
        assert_eq!(body["message"], "missing required header `x-mode`");

        let missing_cookie = client
            .post(&url)
            .header("x-mode", "ok")
            .json(&serde_json::json!({"id": 1, "name": "Fido"}))
            .send()
            .await
            .unwrap();
        let body = rejection_body(
            missing_cookie,
            reqwest::StatusCode::BAD_REQUEST,
            "missing_cookie",
        )
        .await;
        assert_eq!(body["message"], "missing required cookie `session`");

        let invalid_query_url = format!("http://{addr}/rejection-probe/1?limit=invalid");
        let invalid_query = rejection_probe_request(&client, &invalid_query_url, "ok")
            .json(&serde_json::json!({"id": 1, "name": "Fido"}))
            .send()
            .await
            .unwrap();
        let body = rejection_body(
            invalid_query,
            reqwest::StatusCode::BAD_REQUEST,
            "invalid_query",
        )
        .await;
        assert!(
            body["message"]
                .as_str()
                .unwrap()
                .starts_with("invalid query string:")
        );

        let invalid_path_url = format!("http://{addr}/rejection-probe/invalid?limit=1");
        let invalid_path = rejection_probe_request(&client, &invalid_path_url, "ok")
            .json(&serde_json::json!({"id": 1, "name": "Fido"}))
            .send()
            .await
            .unwrap();
        let body =
            rejection_body(invalid_path, reqwest::StatusCode::BAD_REQUEST, "invalid_path").await;
        assert_eq!(body["message"], "invalid path parameter");

        let internal = rejection_probe_request(&client, &url, "internal")
            .json(&serde_json::json!({"id": 1, "name": "Fido"}))
            .send()
            .await
            .unwrap();
        let body = rejection_body(
            internal,
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
        )
        .await;
        assert_eq!(body["message"], "Internal Server Error");
        assert!(!body.to_string().contains("private internal detail"));
    }
}
