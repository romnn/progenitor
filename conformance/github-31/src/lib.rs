//! Generated github-31 API client — conformance crate.
//!
//! This client is one of the corpus monsters (hundreds of thousands of
//! generated lines); building it takes rustc a long time on stable. The
//! nightly parallel frontend helps:
//! `RUSTFLAGS="-Z threads=8" cargo +nightly check -p conformance-github-31`.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    conformance_support::assert_send_sync!(Client);
    conformance_support::assert_display!(types::ReactionContent);
    conformance_support::assert_from_str!(types::ReactionContent);

    #[test]
    fn client_constructs() {
        let client = Client::new("https://example.invalid");
        assert_eq!(client.baseurl(), "https://example.invalid");
    }

    #[test]
    fn reaction_rollup_roundtrip() {
        conformance_support::roundtrip!(
            types::ReactionRollup,
            conformance_support::serde_json::json!({
                "+1": 5,
                "-1": 0,
                "confused": 1,
                "eyes": 0,
                "heart": 3,
                "hooray": 0,
                "laugh": 0,
                "rocket": 2,
                "total_count": 11,
                "url": "https://api.github.com/repos/owner/repo/issues/1/reactions"
            })
        );
    }

    #[test]
    fn reaction_content_wire_names() {
        // +1 and -1 are renamed variants — verify the serde rename roundtrips correctly
        conformance_support::assert_wire_enum!(
            types::ReactionContent,
            "+1" => types::ReactionContent::plus1,
            "-1" => types::ReactionContent::minus1,
            "laugh" => types::ReactionContent::Laugh,
            "heart" => types::ReactionContent::Heart,
        );
    }

    #[test]
    fn org_rules_variants() {
        conformance_support::assert_union_variants!(
            types::OrgRules,
            conformance_support::serde_json::json!({"type": "creation"}) => types::OrgRules::Creation,
            conformance_support::serde_json::json!({"type": "deletion"}) => types::OrgRules::Deletion,
        );
    }

    #[test]
    fn unknown_reaction_rejected() {
        conformance_support::assert_rejects!(
            types::ReactionContent,
            conformance_support::serde_json::json!("clapping"),
            "ReactionContent has no variant for unknown wire value"
        );
    }
}

#[cfg(test)]
mod example_tests {
    include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
}
