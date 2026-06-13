//! Generated Anthropic API client — conformance crate.
//!
//! Demonstrates using progenitor with the Anthropic API and pins, via
//! the tests below, properties of the generated code that a consumer
//! relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    conformance_support::assert_send_sync!(Client);
    conformance_support::assert_display!(types::StopReason);
    conformance_support::assert_from_str!(types::StopReason);

    #[test]
    fn client_constructs() {
        let client = Client::new("https://api.anthropic.com");
        assert_eq!(client.baseurl(), "https://api.anthropic.com");
    }

    #[test]
    fn usage_roundtrip() {
        conformance_support::roundtrip!(
            types::Usage,
            conformance_support::serde_json::json!({
                "input_tokens": 150,
                "output_tokens": 75
            })
        );
    }

    #[test]
    fn stop_reason_wire_names() {
        conformance_support::assert_wire_enum!(
            types::StopReason,
            "end_turn" => types::StopReason::EndTurn,
            "max_tokens" => types::StopReason::MaxTokens,
            "tool_use" => types::StopReason::ToolUse,
        );
    }

    #[test]
    fn content_block_variants() {
        conformance_support::assert_union_variants!(
            types::ContentBlock,
            conformance_support::serde_json::json!({
                "type": "thinking",
                "signature": "sig123",
                "thinking": "I am reasoning through the problem"
            }) => types::ContentBlock::Thinking { .. },
            conformance_support::serde_json::json!({
                "type": "redacted_thinking",
                "data": "redacted_data"
            }) => types::ContentBlock::RedactedThinking { .. },
        );
    }

    #[test]
    fn unknown_stop_reason_rejected() {
        conformance_support::assert_rejects!(
            types::StopReason,
            conformance_support::serde_json::json!("hallucinated"),
            "StopReason has no variant for unknown wire value"
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
    async fn models_list_sends_get_to_v1_models() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/v1/models");
                then.status(200).json_body(serde_json::json!({
                    "data": [],
                    "first_id": null,
                    "has_more": false,
                    "last_id": null
                }));
            })
            .await;
        let client = Client::new(&server.base_url());
        let result = client.models_list(None, None, None, None, None, None).await;
        mock.assert_async().await;
        assert!(result.is_ok(), "expected 200 success from mock");
    }

    #[conformance_support::tokio::test]
    async fn models_get_substitutes_model_id_in_path() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/v1/models/claude-opus-4-6");
                then.status(200).json_body(serde_json::json!({
                    "capabilities": null,
                    "created_at": "2024-01-01T00:00:00Z",
                    "display_name": "Claude Opus 4.6",
                    "id": "claude-opus-4-6",
                    "max_input_tokens": null,
                    "max_tokens": null,
                    "type": "api_error"
                }));
            })
            .await;
        let client = Client::new(&server.base_url());
        let _ = client.models_get("claude-opus-4-6", None, None, None).await;
        mock.assert_async().await;
    }

    #[conformance_support::tokio::test]
    async fn models_list_returns_error_on_401() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/v1/models");
                then.status(401).json_body(serde_json::json!({
                    "error": {
                        "type": "authentication_error",
                        "message": "Invalid API key"
                    },
                    "request_id": null,
                    "type": "api_error"
                }));
            })
            .await;
        let client = Client::new(&server.base_url());
        let result = client.models_list(None, None, None, None, None, None).await;
        mock.assert_async().await;
        match result {
            Err(Error::ErrorResponse(resp)) => {
                let body = resp.into_inner();
                assert!(
                    matches!(body.error, types::Error::AuthenticationError { .. }),
                    "expected AuthenticationError variant"
                );
            }
            _ => panic!("expected ErrorResponse variant"),
        }
    }
}
