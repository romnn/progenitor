//! Generated Perplexity (Sonar) API client — conformance crate.
//!
//! Demonstrates using progenitor with the Perplexity API and pins, via
//! the tests below, properties of the generated code that a consumer
//! relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let client = Client::new("https://api.perplexity.ai");
        assert_eq!(client.baseurl(), "https://api.perplexity.ai");
    }

    #[test]
    fn chat_request_type_round_trips() {
        // The chat-completions request is the core type of this API; it
        // must deserialize from the documented wire shape, with optional
        // fields generated as Options that stay off the wire when unset.
        let request: types::ApiChatCompletionsRequest = serde_json::from_value(serde_json::json!({
            "model": "sonar",
            "messages": [{"role": "user", "content": "hello"}],
        }))
        .expect("documented request shape deserializes");
        let value = serde_json::to_value(&request).expect("serializes");
        assert_eq!(value["model"], "sonar");
        assert!(
            value.get("disable_search").is_none(),
            "unset optional fields must not serialize"
        );
    }

    #[test]
    fn finish_reason_wire_names() {
        conformance_support::assert_wire_enum!(
            types::ChoiceFinishReason,
            "stop" => types::ChoiceFinishReason::Stop,
            "length" => types::ChoiceFinishReason::Length,
        );
    }

    #[test]
    fn unknown_chat_role_rejected() {
        conformance_support::assert_rejects!(
            types::ChatMessageRole,
            conformance_support::serde_json::json!("moderator"),
            "ChatMessageRole has no variant for unknown wire value"
        );
    }
}

#[cfg(test)]
mod example_tests {
    include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
}
