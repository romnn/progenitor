//! Generated Petstore 3.1 API client — conformance crate.
//!
//! Hermetic crate: spec is committed in this directory as petstore-31.yaml.
//! Used as a fast always-asserted PR-CI gate; the tiny spec ensures a quick
//! cold build and any regression in 3.1 baseline handling surfaces immediately.

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

#[cfg(test)]
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
    use progenitor_server::{Request, Response, ServerError};
    use std::sync::Mutex;

    #[derive(Default)]
    struct MyPets {
        store: Mutex<Vec<types::Pet>>,
    }

    struct BadStatus;

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
                None => Err(ServerError::api(
                    progenitor_server::codegen::http::StatusCode::NOT_FOUND,
                    types::Error {
                        code: 404,
                        message: "not found".to_string(),
                    },
                )),
            }
        }
    }

    #[async_trait]
    impl server::Petstore for BadStatus {
        async fn list_pets(
            &self,
            _request: Request<server::ListPetsRequest>,
        ) -> Result<Response<types::Pets>, server::ListPetsError> {
            Ok(Response::new(types::Pets(Vec::new()))
                .with_status(progenitor_server::codegen::http::StatusCode::CREATED))
        }

        async fn create_pets(
            &self,
            _request: Request<server::CreatePetsRequest>,
        ) -> Result<Response<()>, server::CreatePetsError> {
            Ok(Response::new(()))
        }

        async fn show_pet_by_id(
            &self,
            _request: Request<server::ShowPetByIdRequest>,
        ) -> Result<Response<types::Pet>, server::ShowPetByIdError> {
            Ok(Response::new(types::Pet {
                id: 1,
                name: "Fido".to_string(),
                tag: None,
            }))
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
                let body = resp.into_inner();
                assert_eq!(body.code, 404);
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
    async fn undeclared_success_status_becomes_500() {
        let addr = spawn_service(BadStatus).await;
        let response = reqwest::Client::new()
            .get(format!("http://{addr}/pets"))
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
