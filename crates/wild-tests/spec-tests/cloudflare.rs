//! Behavioral spot-checks against the client generated from the
//! Cloudflare spec, pinned to the constructs that used to break
//! generation:
//!
//! 1. `firewall_components-schemas-mode`: an untyped enum (no `type`
//!    keyword) carrying a stale `maxLength: 12` even though the
//!    documented value `managed_challenge` is 17 characters long. The
//!    generator must trust the enum and ignore the unsatisfiable
//!    length bound.
//! 2. `rulesets_ResponseRule = allOf [rulesets_RequestRule, {required:
//!    [id, expression, action, ref, enabled]}]`: refinement of a
//!    20-variant oneOf+discriminator(action) union. The request union
//!    must keep all of its variants and route payloads by `action`,
//!    and the refined response union must both accept conforming
//!    payloads and enforce the extra `required`.
//! 3. Workers KV value PUT: `application/octet-stream` body whose
//!    schema is `anyOf [string, binary string]` — the client method
//!    must take a raw `Into<reqwest::Body>` body, not a JSON type.
//!
//! Every rule payload carries `version` and `last_updated` because the
//! shared base `rulesets_Rule` lists them in `required` (they are
//! readOnly response bookkeeping, but the spec requires them on every
//! rule variant).

use std::str::FromStr;

use wild_cloudflare::types;

fn managed_challenge_violates_the_stale_max_length() {
    // The spec says maxLength: 12 on the very enum that documents this
    // value; honoring it would make the value unrepresentable.
    assert!("managed_challenge".len() > 12);
}

#[test]
fn firewall_mode_enum_accepts_managed_challenge_despite_stale_max_length() {
    managed_challenge_violates_the_stale_max_length();

    let parsed = types::FirewallComponentsSchemasMode::from_str("managed_challenge")
        .expect("FromStr must accept the documented value");
    assert!(matches!(
        parsed,
        types::FirewallComponentsSchemasMode::ManagedChallenge
    ));

    let deserialized: types::FirewallComponentsSchemasMode =
        serde_json::from_str(r#""managed_challenge""#).expect("serde must accept the wire value");
    assert!(matches!(
        deserialized,
        types::FirewallComponentsSchemasMode::ManagedChallenge
    ));

    // The wire form must survive a round-trip unchanged.
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
        assert_eq!(parsed.to_string(), wire, "Display must emit the wire form");

        let json = format!("\"{wire}\"");
        let deserialized: types::FirewallComponentsSchemasMode =
            serde_json::from_str(&json).unwrap_or_else(|err| panic!("`{wire}` via serde: {err}"));
        assert_eq!(serde_json::to_string(&deserialized).unwrap(), json);
    }
}

#[test]
fn firewall_mode_enum_rejects_undocumented_values() {
    assert!(types::FirewallComponentsSchemasMode::from_str("bogus").is_err());
    assert!(types::FirewallComponentsSchemasMode::try_from("bogus").is_err());
    assert!(serde_json::from_str::<types::FirewallComponentsSchemasMode>(r#""bogus""#).is_err());
    // Wire values are case-sensitive.
    assert!(types::FirewallComponentsSchemasMode::from_str("MANAGED_CHALLENGE").is_err());
    // A value that satisfies the stale maxLength but is not in the enum
    // must still be rejected: the enum, not the length bound, decides.
    assert!(types::FirewallComponentsSchemasMode::from_str("ban").is_err());
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

fn skip_rule_payload() -> serde_json::Value {
    // Skip's action_parameters object has minProperties: 1 over
    // {phase, phases, products, rules, ruleset, rulesets}; the
    // `ruleset: "current"` form is the smallest documented shape.
    serde_json::json!({
        "action": "skip",
        "action_parameters": {
            "ruleset": "current"
        },
        "expression": "true",
        "last_updated": "2014-01-01T05:20:00.123123Z",
        "version": "2"
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
    assert_eq!(
        block.expression.as_deref().map(String::as_str),
        Some("ip.src eq 1.2.3.4")
    );

    // The block-specific action_parameters.response must not be
    // silently dropped: every documented field survives a round-trip.
    let round_tripped = serde_json::to_value(&rule).expect("serializes");
    assert_eq!(round_tripped, block_rule_payload());

    let response = block
        .action_parameters
        .as_ref()
        .and_then(|params| params.response.as_ref())
        .expect("action_parameters.response must be preserved");
    assert_eq!(response.status_code, 403);
    assert_eq!(response.content.as_str(), "you have been blocked");
    assert_eq!(response.content_type.as_str(), "text/plain");
}

#[test]
fn request_rule_union_routes_skip_payload_to_skip_variant() {
    let rule: types::RulesetsRequestRule =
        serde_json::from_value(skip_rule_payload()).expect("skip payload deserializes");

    let types::RulesetsRequestRule::SkipRule(skip) = &rule else {
        panic!("skip payload must pick the SkipRule variant, got {rule:?}");
    };
    assert!(matches!(
        skip.action,
        Some(types::RulesetsSkipRuleAction::Skip)
    ));

    // The skip-specific action_parameters must come through typed, not
    // be swallowed by some catch-all object.
    let params = skip
        .action_parameters
        .as_ref()
        .expect("skip action_parameters must be preserved");
    assert!(matches!(
        params.ruleset,
        Some(types::RulesetsSkipRuleset::Current)
    ));

    // The union must discriminate: an identical base shape with a
    // different action lands in a different variant.
    let block: types::RulesetsRequestRule =
        serde_json::from_value(block_rule_payload()).expect("block payload deserializes");
    assert_ne!(
        std::mem::discriminant(&block),
        std::mem::discriminant(&rule)
    );
}

fn response_block_rule_payload() -> serde_json::Value {
    let mut payload = block_rule_payload();
    let object = payload.as_object_mut().expect("payload is an object");
    // The fields the allOf refinement promotes to required.
    object.insert(
        "id".into(),
        serde_json::json!("3a03d665bac047339bb530ecb439a90d"),
    );
    object.insert("ref".into(), serde_json::json!("my_ref"));
    object.insert("enabled".into(), serde_json::json!(true));
    payload
}

#[test]
fn response_rule_block_payload_deserializes_with_refined_fields() {
    let rule: types::RulesetsResponseRule =
        serde_json::from_value(response_block_rule_payload())
            .expect("refined block payload deserializes");

    // None of the refined-required fields may be dropped on the floor.
    let round_tripped = serde_json::to_value(&rule).expect("serializes");
    assert_eq!(round_tripped, response_block_rule_payload());
    assert_eq!(round_tripped["id"], "3a03d665bac047339bb530ecb439a90d");
    assert_eq!(round_tripped["ref"], "my_ref");
    assert_eq!(round_tripped["enabled"], true);
    assert_eq!(round_tripped["action"], "block");
}

#[test]
fn response_rule_enforces_refined_required_fields() {
    // Without id/ref/enabled the payload is a fine *request* rule…
    let request: Result<types::RulesetsRequestRule, _> =
        serde_json::from_value(block_rule_payload());
    assert!(
        request.is_ok(),
        "request union must not require the response-only fields"
    );

    // …but the response refinement must reject it.
    let response: Result<types::RulesetsResponseRule, _> =
        serde_json::from_value(block_rule_payload());
    assert!(
        response.is_err(),
        "response union must enforce required [id, expression, action, ref, enabled]"
    );
}

#[test]
fn kv_value_put_takes_a_raw_octet_stream_body() {
    // The PUT body is `application/octet-stream` with schema
    // `anyOf [string, binary string]`; the generated method is generic
    // over `B: Into<reqwest::Body>` rather than taking a JSON type.
    // Building the futures (without polling them — progenitor request
    // futures do no I/O until awaited) proves both text and arbitrary
    // binary bodies are accepted.
    let client = wild_cloudflare::Client::new("https://api.cloudflare.invalid");
    let account_id =
        types::WorkersKvIdentifier::from_str("023e105f4ecef8ad9ca31a8372d0c353").unwrap();
    let namespace_id =
        types::WorkersKvNamespaceIdentifier::from_str("0f2ac74b498b48028cb68387c421e279").unwrap();
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
