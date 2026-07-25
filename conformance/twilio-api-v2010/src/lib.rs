//! Generated Twilio API v2010 client — conformance crate.
//!
//! Demonstrates using progenitor with the Twilio core REST API and
//! pins, via the tests below, properties of the generated code that a
//! consumer relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    conformance_support::assert_send_sync!(Client);
    conformance_support::assert_display!(types::MessageEnumStatus, types::MessageEnumDirection);
    conformance_support::assert_from_str!(types::MessageEnumStatus, types::MessageEnumDirection);

    #[test]
    fn client_constructs() {
        let client = Client::new("https://api.twilio.com");
        assert_eq!(client.baseurl(), "https://api.twilio.com");
    }

    #[test]
    fn message_resource_round_trips() {
        // The Message resource is the core type of this API. SIDs are
        // pattern-validated newtypes (^AC…$/^(SM|MM)…$), so the sample
        // values must be 34-char Twilio-shaped SIDs or deserialization
        // fails.
        let message: types::ApiV2010AccountMessage = serde_json::from_value(serde_json::json!({
            "account_sid": "ACaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "sid": "SMb7c0a2ce80504485a6f653a7110836f5",
            "body": "Hello there",
            "direction": "outbound-api",
            "status": "delivered",
            "from": "+14155552345",
            "to": "+14155552671",
            "num_segments": "1"
        }))
        .expect("documented message shape deserializes");
        assert_eq!(message.body.as_deref(), Some("Hello there"));
        assert_eq!(
            message.direction,
            Some(types::MessageEnumDirection::OutboundApi)
        );

        let value = serde_json::to_value(&message).expect("serializes");
        assert_eq!(value["sid"], "SMb7c0a2ce80504485a6f653a7110836f5");
        assert!(
            value.get("price").is_none(),
            "unset optional fields must not serialize"
        );
    }

    #[test]
    fn message_status_enum_parses_wire_names() {
        // Direction uses hyphenated wire names while status uses
        // snake_case; both renames must survive generation.
        let status: types::MessageEnumStatus =
            serde_json::from_value(serde_json::json!("partially_delivered"))
                .expect("status variant parses");
        assert_eq!(status, types::MessageEnumStatus::PartiallyDelivered);
        assert_eq!(status.to_string(), "partially_delivered");
        assert_eq!(
            "outbound-call"
                .parse::<types::MessageEnumDirection>()
                .expect("hyphenated direction parses"),
            types::MessageEnumDirection::OutboundCall
        );
    }

    #[test]
    fn create_message_request_uses_pascal_case_params() {
        // Twilio's form-urlencoded request bodies use PascalCase
        // parameter names; the generated request type must keep those
        // serde renames and keep unset optionals off the wire.
        let request: types::CreateMessageRequest = serde_json::from_value(serde_json::json!({
            "To": "+14155552671",
            "From": "+14155552345",
            "Body": "Hi there"
        }))
        .expect("documented create-message shape deserializes");
        assert_eq!(request.to, "+14155552671");

        let value = serde_json::to_value(&request).expect("serializes");
        assert_eq!(value["To"], "+14155552671");
        assert_eq!(value["Body"], "Hi there");
        assert!(
            value.get("MediaUrl").is_none() && value.get("StatusCallback").is_none(),
            "unset optional form parameters must not serialize"
        );
    }

    #[test]
    fn unknown_account_status_rejected() {
        conformance_support::assert_rejects!(
            types::AccountEnumStatus,
            conformance_support::serde_json::json!("pending"),
            "AccountEnumStatus has no variant for unknown wire value"
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
    async fn list_account_sends_get_to_accounts_json() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/2010-04-01/Accounts.json");
                then.status(200).json_body(serde_json::json!({}));
            })
            .await;
        let client = Client::new(&server.base_url());
        let result = client.list_account(None, None, None, None, None).await;
        mock.assert_async().await;
        assert!(result.is_ok(), "expected 200 success from mock");
    }
}

/// Generated axum server stub: the service trait and its `{Api}Server` adapter.
#[allow(clippy::all)]
pub mod server {
    include!(concat!(env!("OUT_DIR"), "/server.rs"));
}
