//! Generated Meilisearch API client — conformance crate.
//!
//! Demonstrates using progenitor with the Meilisearch API and pins, via
//! the tests below, properties of the generated code that a consumer
//! relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    conformance_support::assert_send_sync!(Client);
    conformance_support::assert_display!(types::MatchingStrategy);
    conformance_support::assert_from_str!(types::MatchingStrategy);

    #[test]
    fn client_constructs() {
        let client = Client::new("http://localhost:7700");
        assert_eq!(client.baseurl(), "http://localhost:7700");
    }

    #[test]
    fn index_settings_round_trip() {
        // The index-settings document is the central resource of this
        // API: every field is optional and camelCase on the wire.
        let settings: types::SettingsUnchecked = serde_json::from_value(serde_json::json!({
            "displayedAttributes": ["title", "overview"],
            "filterableAttributes": ["genres"],
            "rankingRules": ["words", "typo", "proximity"],
            "searchCutoffMs": 150,
        }))
        .expect("documented settings shape deserializes");
        assert_eq!(settings.search_cutoff_ms, Some(150));
        let value = serde_json::to_value(&settings).expect("serializes");
        assert_eq!(value["displayedAttributes"][0], "title");
        assert!(
            value.get("stopWords").is_none(),
            "unset optional fields must not serialize"
        );
    }

    #[test]
    fn index_create_request_round_trips() {
        let request: types::IndexCreateRequest = serde_json::from_value(serde_json::json!({
            "uid": "movies",
        }))
        .expect("minimal index creation body deserializes");
        assert_eq!(request.uid.0, "movies");
        let value = serde_json::to_value(&request).expect("serializes");
        assert!(
            value.get("primaryKey").is_none(),
            "unset primaryKey must not serialize"
        );
    }

    #[test]
    fn filterable_attributes_rule_parses_both_variants() {
        // filterableAttributes entries are a oneOf of plain field name
        // or a pattern object; the spec nests the patterns list inside
        // an `attributePatterns` object.
        let rules: Vec<types::FilterableAttributesRule> =
            serde_json::from_value(serde_json::json!([
                "genres",
                {"attributePatterns": {"patterns": ["release_*"]}},
            ]))
            .expect("both oneOf branches deserialize");
        assert!(matches!(&rules[0], types::FilterableAttributesRule::String(s) if s == "genres"));
        assert!(matches!(
            &rules[1],
            types::FilterableAttributesRule::FilterableAttributesPatterns(p)
                if p.attribute_patterns.patterns == ["release_*"]
        ));
    }

    #[test]
    fn unknown_matching_strategy_rejected() {
        conformance_support::assert_rejects!(
            types::MatchingStrategy,
            conformance_support::serde_json::json!("fuzzy"),
            "MatchingStrategy has no variant for unknown wire value"
        );
    }
}

#[cfg(test)]
mod example_tests {
    include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
}
