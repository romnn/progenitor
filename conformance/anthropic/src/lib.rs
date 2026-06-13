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
