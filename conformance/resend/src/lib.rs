//! Generated Resend API client — conformance crate.
//!
//! Demonstrates using progenitor with the Resend email API and pins, via
//! the tests below, properties of the generated code that a consumer
//! relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    conformance_support::assert_send_sync!(Client);
    conformance_support::assert_display!(types::CreateApiKeyRequestPermission);
    conformance_support::assert_from_str!(types::CreateApiKeyRequestPermission);

    #[test]
    fn client_constructs() {
        let client = Client::new("https://api.resend.com");
        assert_eq!(client.baseurl(), "https://api.resend.com");
    }

    #[test]
    fn send_email_request_round_trips() {
        // The send-email request is the core type of this API; it must
        // deserialize from the documented wire shape, with optional fields
        // generated as Options (and defaulted Vecs/Maps) that stay off the
        // wire when unset.
        let request: types::SendEmailRequest = serde_json::from_value(serde_json::json!({
            "from": "Acme <onboarding@resend.dev>",
            "to": ["delivered@resend.dev"],
            "subject": "hello world",
            "html": "<p>it works!</p>",
        }))
        .expect("documented request shape deserializes");
        let value = serde_json::to_value(&request).expect("serializes");
        assert_eq!(value["from"], "Acme <onboarding@resend.dev>");
        assert_eq!(value["subject"], "hello world");
        for unset in [
            "bcc",
            "cc",
            "text",
            "scheduled_at",
            "attachments",
            "tags",
            "headers",
        ] {
            assert!(
                value.get(unset).is_none(),
                "unset optional field `{unset}` must not serialize"
            );
        }
    }

    #[test]
    fn to_field_accepts_string_or_array() {
        // `to` is a oneOf of string | string[] in the spec; the generated
        // untagged enum must accept both wire shapes.
        let single: types::SendEmailRequestTo =
            serde_json::from_value(serde_json::json!("delivered@resend.dev")).expect("string");
        assert!(matches!(single, types::SendEmailRequestTo::String(_)));

        let many: types::SendEmailRequestTo =
            serde_json::from_value(serde_json::json!(["a@resend.dev", "b@resend.dev"]))
                .expect("array");
        assert!(matches!(
            many,
            types::SendEmailRequestTo::Array(ref items) if items.len() == 2
        ));
    }

    #[test]
    fn unknown_api_key_permission_rejected() {
        conformance_support::assert_rejects!(
            types::CreateApiKeyRequestPermission,
            conformance_support::serde_json::json!("admin"),
            "CreateApiKeyRequestPermission has no variant for unknown wire value"
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
    async fn get_emails_sends_get_to_emails() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/emails");
                then.status(200).json_body(serde_json::json!({}));
            })
            .await;
        let client = Client::new(&server.base_url());
        let result = client.get_emails(None, None, None).await;
        mock.assert_async().await;
        assert!(result.is_ok(), "expected 200 success from mock");
    }
}
