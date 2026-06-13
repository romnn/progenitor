//! Generated openai API client — conformance crate.
//!
//! This client is one of the corpus monsters (hundreds of thousands of
//! generated lines); building it takes rustc a long time on stable. The
//! nightly parallel frontend helps:
//! `RUSTFLAGS="-Z threads=8" cargo +nightly check -p conformance-openai`.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    conformance_support::assert_send_sync!(Client);
    conformance_support::assert_display!(types::ChatCompletionRole);
    conformance_support::assert_from_str!(types::ChatCompletionRole);

    #[test]
    fn client_constructs() {
        let client = Client::new("https://example.invalid");
        assert_eq!(client.baseurl(), "https://example.invalid");
    }

    #[test]
    fn completion_usage_roundtrip() {
        let usage = conformance_support::roundtrip!(
            types::CompletionUsage,
            conformance_support::serde_json::json!({
                "completion_tokens": 50,
                "prompt_tokens": 100,
                "total_tokens": 150
            })
        );
        conformance_support::assert_off_wire!(
            usage,
            "completion_tokens_details",
            "prompt_tokens_details"
        );
    }

    #[test]
    fn chat_completion_role_wire_names() {
        conformance_support::assert_wire_enum!(
            types::ChatCompletionRole,
            "user" => types::ChatCompletionRole::User,
            "assistant" => types::ChatCompletionRole::Assistant,
            "tool" => types::ChatCompletionRole::Tool,
        );
    }

    #[test]
    fn annotation_variants() {
        conformance_support::assert_union_variants!(
            types::Annotation,
            conformance_support::serde_json::json!({
                "type": "file_citation",
                "file_id": "file-abc123",
                "filename": "document.pdf",
                "index": 0
            }) => types::Annotation::FileCitation { .. },
            conformance_support::serde_json::json!({
                "type": "url_citation",
                "end_index": 100,
                "start_index": 0,
                "title": "Example Page",
                "url": "https://example.com"
            }) => types::Annotation::UrlCitation { .. },
        );
    }

    #[test]
    fn unknown_chat_role_rejected() {
        conformance_support::assert_rejects!(
            types::ChatCompletionRole,
            conformance_support::serde_json::json!("invalid_role"),
            "ChatCompletionRole has no variant for unknown wire value"
        );
    }
}

#[cfg(test)]
mod example_tests {
    include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
}
