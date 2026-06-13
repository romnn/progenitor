//! Generated Svix (webhooks) API client — conformance crate.
//!
//! Demonstrates using progenitor with the Svix API and pins, via the
//! tests below, properties of the generated code that a consumer
//! relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    conformance_support::assert_send_sync!(Client);
    conformance_support::assert_display!(types::MessageStatusText);
    conformance_support::assert_from_str!(types::MessageStatusText);

    #[test]
    fn client_constructs() {
        let client = Client::new("https://api.svix.com");
        assert_eq!(client.baseurl(), "https://api.svix.com");
    }

    #[test]
    fn message_in_round_trips() {
        // MessageIn is the core request type (create-message). camelCase
        // wire names map to snake_case fields, and the spec's
        // `payloadRetentionPeriod` default of 90 is materialized on
        // deserialize (it has a schema default, so it is not skipped on
        // re-serialization).
        let message: types::MessageIn = serde_json::from_value(serde_json::json!({
            "eventType": "user.created",
            "payload": {"username": "test_user"},
        }))
        .expect("documented request shape deserializes");
        assert_eq!(&*message.event_type, "user.created");

        let value = serde_json::to_value(&message).expect("serializes");
        assert_eq!(value["eventType"], "user.created");
        assert_eq!(value["payloadRetentionPeriod"], 90);
        assert!(
            value.get("channels").is_none(),
            "unset optional fields must not serialize"
        );
    }

    #[test]
    fn message_out_deserializes() {
        // MessageOut is the central resource; pin the required id and
        // RFC 3339 timestamp handling via chrono.
        let message: types::MessageOut = serde_json::from_value(serde_json::json!({
            "eventType": "user.created",
            "id": "msg_1srOrx2ZWZBpBUvZwXKQmoEYga2",
            "payload": {"username": "test_user"},
            "timestamp": "2026-01-01T00:00:00Z",
        }))
        .expect("documented resource shape deserializes");
        assert_eq!(&*message.id, "msg_1srOrx2ZWZBpBUvZwXKQmoEYga2");
        // 2026-01-01T00:00:00Z as a Unix epoch; the workspace chrono has
        // formatting features off, so compare numerically.
        assert_eq!(message.timestamp.timestamp(), 1_767_225_600);
    }

    #[test]
    fn message_status_text_variant_parses() {
        let status: types::MessageStatusText =
            serde_json::from_value(serde_json::json!("sending")).expect("known variant parses");
        assert!(matches!(status, types::MessageStatusText::Sending));
        assert_eq!(status.to_string(), "sending");
    }

    #[test]
    fn unknown_message_status_rejected() {
        conformance_support::assert_rejects!(
            types::MessageStatusText,
            conformance_support::serde_json::json!("queued"),
            "MessageStatusText has no variant for unknown wire value"
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
    async fn v1_application_list_sends_get_to_api_v1_app() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/api/v1/app");
                then.status(200).json_body(serde_json::json!({
                    "data": [],
                    "done": true,
                    "iterator": null
                }));
            })
            .await;
        let client = Client::new(&server.base_url());
        let result = client
            .v1_application_list(None, None, None, None, None, None)
            .await;
        mock.assert_async().await;
        assert!(result.is_ok(), "expected 200 success from mock");
    }
}
