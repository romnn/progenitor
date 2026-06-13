//! Generated Hetzner Cloud API client — conformance crate.
//!
//! Demonstrates using progenitor with the Hetzner Cloud API and pins, via
//! the tests below, properties of the generated code that a consumer
//! relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    conformance_support::assert_send_sync!(Client);
    conformance_support::assert_display!(types::ActionStatus);
    conformance_support::assert_from_str!(types::ActionStatus);

    #[test]
    fn client_constructs() {
        let client = Client::new("https://api.hetzner.cloud/v1");
        assert_eq!(client.baseurl(), "https://api.hetzner.cloud/v1");
    }

    #[test]
    fn create_server_request_round_trips() {
        // POST /servers is the central operation of this API; the minimal
        // documented body (name, server_type, image) must deserialize, and
        // unset optionals must stay off the wire when re-serialized.
        let request: types::CreateServerRequest = serde_json::from_value(serde_json::json!({
            "name": "my-server",
            "server_type": "cx22",
            "image": "ubuntu-24.04",
        }))
        .expect("minimal documented request shape deserializes");
        assert_eq!(request.name, "my-server");
        // The spec defaults start_after_create to true; the generated type
        // materializes that default rather than using an Option.
        assert!(request.start_after_create);

        let value = serde_json::to_value(&request).expect("serializes");
        assert_eq!(value["server_type"], "cx22");
        assert!(
            value.get("user_data").is_none(),
            "unset optional fields must not serialize"
        );
        assert!(
            value.get("labels").is_none(),
            "empty label maps must not serialize"
        );
    }

    #[test]
    fn server_status_enum_variant_parses() {
        // The server lifecycle status enum is the spec's core state machine.
        let status: types::GetServerResponseServerStatus =
            serde_json::from_value(serde_json::json!("running")).expect("variant parses");
        assert!(matches!(
            status,
            types::GetServerResponseServerStatus::Running
        ));
        assert_eq!(status.to_string(), "running");
    }

    #[test]
    fn unknown_action_status_rejected() {
        conformance_support::assert_rejects!(
            types::ActionStatus,
            conformance_support::serde_json::json!("pending"),
            "ActionStatus has no variant for unknown wire value"
        );
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
mod operation_tests {
    use super::*;
    use conformance_support::httpmock::MockServer;

    #[conformance_support::tokio::test]
    async fn get_actions_sends_get_to_actions() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/actions");
                then.status(200).json_body(serde_json::json!({"actions": []}));
            })
            .await;
        let client = Client::new(&server.base_url());
        let result = client.get_actions(&vec![]).await;
        mock.assert_async().await;
        assert!(result.is_ok(), "expected 200 success from mock");
    }
}
