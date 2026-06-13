//! Generated PagerDuty API client — conformance crate.
//!
//! Tests pin the two repairs that used to break generation:
//! 1. A Response misfiled under components.requestBodies
//! 2. Deep JSON-pointer refs hoisted to named shared components

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    conformance_support::assert_send_sync!(Client);
    conformance_support::assert_display!(types::AlertStatus);
    conformance_support::assert_from_str!(types::AlertStatus);

    #[test]
    fn put_cache_variable_data_200_response_deserializes_string_example() {
        let payload = r#"{
            "cache_variable_data": "Updated - Hello World!",
            "updated_at": "2021-11-18T16:42:01Z"
        }"#;
        let response: types::UpdateExternalDataCacheVarDataOnGlobalOrchResponse =
            serde_json::from_str(payload).expect("PUT 200 body deserializes");

        match &response {
            types::UpdateExternalDataCacheVarDataOnGlobalOrchResponse::Of0(string_data) => {
                assert_eq!(
                    string_data.cache_variable_data.as_str(),
                    "Updated - Hello World!"
                );
            }
            _ => panic!("string payload must select the String Data branch"),
        }
    }

    #[test]
    fn client_constructs() {
        let client = Client::new("https://api.pagerduty.com");
        assert_eq!(client.baseurl(), "https://api.pagerduty.com");
    }

    #[test]
    fn tag_round_trips() {
        let payload = serde_json::json!({
            "id": "ABCDEF1",
            "type": "tag_reference",
            "summary": "Team:Engineering"
        });
        let tag: types::TagReference =
            serde_json::from_value(payload.clone()).expect("tag reference deserializes");
        assert_eq!(tag.id.as_str(), "ABCDEF1");
        assert_eq!(tag.summary.as_deref(), Some("Team:Engineering"));
        let round_tripped = serde_json::to_value(&tag).expect("serializes");
        assert_eq!(round_tripped, payload);
    }

    #[test]
    fn unknown_alert_status_rejected() {
        conformance_support::assert_rejects!(
            types::AlertStatus,
            conformance_support::serde_json::json!("pending"),
            "AlertStatus has no variant for unknown wire value"
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
    async fn list_abilities_sends_get_to_abilities() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/abilities");
                then.status(200).json_body(serde_json::json!({"abilities": []}));
            })
            .await;
        let client = Client::new(&server.base_url());
        let result = client
            .list_abilities("application/json", types::ListAbilitiesContentType::ApplicationJson)
            .await;
        mock.assert_async().await;
        assert!(result.is_ok(), "expected 200 success from mock");
    }
}
