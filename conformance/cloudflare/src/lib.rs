//! Generated Cloudflare API client — conformance crate.
//!
//! Tests pin:
//! 1. firewall_components-schemas-mode: untyped enum with stale maxLength
//! 2. rulesets_ResponseRule: allOf refinement of a discriminated union
//! 3. Workers KV value PUT: octet-stream body → Into<reqwest::Body>

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn firewall_mode_enum_accepts_managed_challenge_despite_stale_max_length() {
        // "managed_challenge" is 17 chars; spec says maxLength: 12. The generator
        // must trust the enum values, not the stale length bound.
        assert!("managed_challenge".len() > 12);

        let parsed = types::FirewallComponentsSchemasMode::from_str("managed_challenge")
            .expect("FromStr must accept the documented value");
        assert!(matches!(
            parsed,
            types::FirewallComponentsSchemasMode::ManagedChallenge
        ));

        let deserialized: types::FirewallComponentsSchemasMode =
            serde_json::from_str(r#""managed_challenge""#).expect("serde must accept it");
        assert_eq!(
            serde_json::to_string(&deserialized).expect("serializes"),
            r#""managed_challenge""#
        );
        assert_eq!(parsed.to_string(), "managed_challenge");
    }

    #[test]
    fn firewall_mode_enum_accepts_every_documented_value() {
        for wire in ["block", "challenge", "js_challenge", "managed_challenge"] {
            let parsed = types::FirewallComponentsSchemasMode::from_str(wire)
                .unwrap_or_else(|err| panic!("`{wire}` must parse: {err}"));
            assert_eq!(parsed.to_string(), wire);
        }
    }

    #[test]
    fn firewall_mode_enum_rejects_undocumented_values() {
        assert!(types::FirewallComponentsSchemasMode::from_str("bogus").is_err());
        assert!(types::FirewallComponentsSchemasMode::from_str("MANAGED_CHALLENGE").is_err());
    }

    fn block_rule_payload() -> serde_json::Value {
        serde_json::json!({
            "action": "block",
            "action_parameters": {
                "response": {
                    "content": "you have been blocked",
                    "content_type": "text/plain",
                    "status_code": 403
                }
            },
            "expression": "ip.src eq 1.2.3.4",
            "last_updated": "2000-01-01T00:00:00Z",
            "version": "1"
        })
    }

    #[test]
    fn request_rule_union_routes_block_payload_to_block_variant() {
        let rule: types::RulesetsRequestRule =
            serde_json::from_value(block_rule_payload()).expect("block payload deserializes");

        let types::RulesetsRequestRule::BlockRule(block) = &rule else {
            panic!("block payload must pick the BlockRule variant, got {rule:?}");
        };
        assert!(matches!(
            block.action,
            Some(types::RulesetsBlockRuleAction::Block)
        ));

        let round_tripped = serde_json::to_value(&rule).expect("serializes");
        assert_eq!(round_tripped, block_rule_payload());
    }

    #[test]
    fn kv_value_put_takes_a_raw_octet_stream_body() {
        let client = Client::new("https://api.cloudflare.invalid");
        let account_id =
            types::WorkersKvIdentifier::from_str("023e105f4ecef8ad9ca31a8372d0c353").unwrap();
        let namespace_id =
            types::WorkersKvNamespaceIdentifier::from_str("0f2ac74b498b48028cb68387c421e279")
                .unwrap();
        let key_name = types::WorkersKvKeyName::from_str("my-key").unwrap();

        let binary_body: Vec<u8> = vec![0x00, 0xff, 0x42];
        let _ = client.workers_kv_namespace_write_key_value_pair_with_metadata(
            &account_id,
            &namespace_id,
            &key_name,
            None,
            None,
            binary_body,
        );
        let _ = client.workers_kv_namespace_write_key_value_pair_with_metadata(
            &account_id,
            &namespace_id,
            &key_name,
            None,
            None,
            String::from("Some Value"),
        );
    }
}

#[cfg(test)]
mod example_tests {
    include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
}

/// Generated axum server stub: the service trait and its `{Api}Server` adapter.
#[allow(clippy::all)]
pub mod server {
    include!(concat!(env!("OUT_DIR"), "/server.rs"));
}
