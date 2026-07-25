//! Generated ClickHouse Cloud API client — conformance crate.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn level_accepts_documented_percentiles() {
        for value in [0.5_f64, 0.9, 0.95, 0.99] {
            let level = types::ClickStackSelectItemLevel::try_from(value)
                .unwrap_or_else(|e| panic!("level {value} must be accepted: {e}"));
            // Deref/From give back the same f64 — no lossy string detour.
            assert_eq!(f64::from(level.clone()), value);
            assert_eq!(*level, value);
        }
    }

    #[test]
    fn level_rejects_undocumented_percentiles() {
        for value in [0.7_f64, 0.0, 1.0, 0.951] {
            assert!(
                types::ClickStackSelectItemLevel::try_from(value).is_err(),
                "level {value} is not in the spec enum and must be rejected"
            );
        }
    }

    #[test]
    fn level_wire_format_is_a_json_number() {
        let item: types::ClickStackSelectItem = serde_json::from_value(serde_json::json!({
            "aggFn": "quantile",
            "valueExpression": "Duration",
            "level": 0.95
        }))
        .expect("a numeric level must deserialize");
        assert_eq!(item.level.as_deref().copied(), Some(0.95));

        let value = serde_json::to_value(&item).expect("serializes");
        assert_eq!(value["level"], serde_json::json!(0.95));
        assert!(value["level"].is_f64(), "level must stay numeric on the wire");
    }

    #[test]
    fn level_rejects_strings_and_bad_numbers_on_the_wire() {
        let err = serde_json::from_value::<types::ClickStackSelectItem>(serde_json::json!({
            "aggFn": "quantile",
            "level": 0.7
        }));
        assert!(err.is_err(), "level 0.7 must be rejected");

        let err = serde_json::from_value::<types::ClickStackSelectItem>(serde_json::json!({
            "aggFn": "quantile",
            "level": "0.5"
        }));
        assert!(err.is_err(), "a string-typed level must be rejected");
    }

    #[test]
    fn select_item_fields_survive_a_round_trip() {
        let input = serde_json::json!({
            "aggFn": "quantile",
            "valueExpression": "Duration",
            "alias": "Request Duration",
            "level": 0.5,
            "where": "service:api",
            "whereLanguage": "lucene"
        });
        let item: types::ClickStackSelectItem =
            serde_json::from_value(input.clone()).expect("deserializes");

        assert_eq!(item.agg_fn, types::ClickStackSelectItemAggFn::Quantile);
        assert_eq!(item.value_expression.as_deref(), Some("Duration"));
        assert_eq!(item.alias.as_deref(), Some("Request Duration"));
        assert_eq!(item.level.as_deref().copied(), Some(0.5));
        assert_eq!(item.where_.as_deref(), Some("service:api"));
        assert_eq!(
            item.where_language,
            Some(types::ClickStackSelectItemWhereLanguage::Lucene)
        );

        let output = serde_json::to_value(&item).expect("serializes");
        for key in ["aggFn", "valueExpression", "alias", "level", "where", "whereLanguage"] {
            assert_eq!(output[key], input[key], "field {key} must survive a round-trip");
        }
    }

    #[test]
    fn api_key_round_trips() {
        let key: types::ApiKey = serde_json::from_value(serde_json::json!({
            "id": "7f31bf98-3b51-45b6-9bd9-c7c0e7e2a3c4",
            "name": "ci-key",
            "state": "disabled",
            "roles": ["admin", "query_endpoints"],
            "keySuffix": "ab12",
            "createdAt": "2026-01-02T03:04:05Z",
            "expireAt": "2027-01-01T00:00:00Z"
        }))
        .expect("deserializes");

        assert_eq!(
            key.id.map(|u| u.to_string()).as_deref(),
            Some("7f31bf98-3b51-45b6-9bd9-c7c0e7e2a3c4")
        );
        assert_eq!(key.name.as_deref(), Some("ci-key"));
        assert_eq!(key.state, Some(types::ApiKeyState::Disabled));
        assert_eq!(
            key.roles,
            vec![
                types::ApiKeyRolesItem::Admin,
                types::ApiKeyRolesItem::QueryEndpoints
            ]
        );
        assert_eq!(key.key_suffix.as_deref(), Some("ab12"));
        assert_eq!(key.expire_at.map(|t| t.timestamp()), Some(1798761600));

        let value = serde_json::to_value(&key).expect("serializes");
        assert_eq!(value["state"], "disabled");
        assert_eq!(value["roles"], serde_json::json!(["admin", "query_endpoints"]));
        assert_eq!(value["expireAt"], "2027-01-01T00:00:00Z");
    }

    #[test]
    fn api_key_state_guards_documented_values() {
        assert_eq!(
            "enabled".parse::<types::ApiKeyState>().ok(),
            Some(types::ApiKeyState::Enabled)
        );
        assert_eq!(
            "disabled".parse::<types::ApiKeyState>().ok(),
            Some(types::ApiKeyState::Disabled)
        );
        assert!("paused".parse::<types::ApiKeyState>().is_err());
        assert!("Enabled".parse::<types::ApiKeyState>().is_err());
    }

    #[test]
    fn api_key_expire_at_accepts_explicit_null() {
        let key: types::ApiKey =
            serde_json::from_value(serde_json::json!({ "name": "min", "expireAt": null }))
                .expect("explicit null expireAt must deserialize");
        assert_eq!(key.expire_at, None);
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
    async fn organization_get_list_sends_get_to_v1_organizations() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/v1/organizations");
                then.status(200).json_body(serde_json::json!({}));
            })
            .await;
        let client = Client::new(&server.base_url());
        let result = client.organization_get_list().await;
        mock.assert_async().await;
        assert!(result.is_ok(), "expected 200 success from mock");
    }
}

/// Generated axum server stub: the service trait and its `{Api}Server` adapter.
#[allow(clippy::all)]
pub mod server {
    include!(concat!(env!("OUT_DIR"), "/server.rs"));
}
