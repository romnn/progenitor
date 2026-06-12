//! Behavioral spot-checks against the client generated from the
//! PagerDuty spec, pinning the two repairs that used to break
//! generation:
//!
//! 1. A Response object misfiled under `components.requestBodies`
//!    (`OrchestrationCacheVariableDataPutResponse`) referenced from the
//!    200-response position of two PUT operations.
//! 2. Deep JSON-pointer refs (`#/components/schemas/Tag/allOf/0`,
//!    `#/components/responses/OrchestrationCacheVariableGetDataResponse/
//!    content/application~1json/schema/oneOf/{0,1,2}`) hoisted to named
//!    components shared by every referencing site.

use wild_pagerduty::types;

/// The misfiled requestBody-as-Response repair: the PUT
/// `/event_orchestrations/{id}/cache_variables/{cache_variable_id}/data`
/// operation must have a 200 response type, and the spec's documented
/// "string data" example must deserialize into it.
#[test]
fn put_cache_variable_data_200_response_deserializes_string_example() {
    let payload = r#"{
        "cache_variable_data": "Updated - Hello World!",
        "updated_at": "2021-11-18T16:42:01Z"
    }"#;
    let response: types::UpdateExternalDataCacheVarDataOnGlobalOrchResponse =
        serde_json::from_str(payload).expect("PUT 200 body deserializes");

    let types::UpdateExternalDataCacheVarDataOnGlobalOrchResponse::Of0(string_data) = response
    else {
        panic!("string payload must select the String Data branch");
    };
    assert_eq!(string_data.cache_variable_data, "Updated - Hello World!");
    let updated_at = string_data.updated_at.expect("updated_at must not be dropped");
    // 2021-11-18T16:42:01Z; chrono is built without `alloc`, so compare epochs.
    assert_eq!(updated_at.timestamp(), 1_637_253_721);

    // Both documented fields must survive a round-trip.
    let value = serde_json::to_value(&string_data).expect("serializes");
    assert_eq!(value["cache_variable_data"], "Updated - Hello World!");
    assert!(value.get("updated_at").is_some(), "updated_at dropped on serialize");
}

/// The repaired response is a oneOf over string/number/boolean data;
/// the untagged enum must discriminate the spec's other two documented
/// examples correctly.
#[test]
fn put_response_discriminates_number_and_boolean_examples() {
    let number: types::UpdateExternalDataCacheVarDataOnGlobalOrchResponse = serde_json::from_str(
        r#"{"cache_variable_data": 85.1, "updated_at": "2021-11-18T16:42:01Z"}"#,
    )
    .expect("number body deserializes");
    let types::UpdateExternalDataCacheVarDataOnGlobalOrchResponse::Of1(number_data) = number
    else {
        panic!("numeric payload must select the Number Data branch");
    };
    assert_eq!(number_data.cache_variable_data, 85.1);

    let boolean: types::UpdateExternalDataCacheVarDataOnGlobalOrchResponse = serde_json::from_str(
        r#"{"cache_variable_data": false, "updated_at": "2021-11-18T16:42:01Z"}"#,
    )
    .expect("boolean body deserializes");
    let types::UpdateExternalDataCacheVarDataOnGlobalOrchResponse::Of2(boolean_data) = boolean
    else {
        panic!("boolean payload must select the Boolean Data branch");
    };
    assert!(!boolean_data.cache_variable_data);
}

/// The misfiled component's oneOf branches were deep-pointer refs into
/// `#/components/responses/OrchestrationCacheVariableGetDataResponse`.
/// The hoisted branch structs must be a single shared component: the
/// same Rust type has to be accepted by both PUT-site response enums,
/// the GET response enum, and the PUT request-body enum. These `From`
/// conversions only compile if all four wrap the identical struct.
#[test]
fn hoisted_oneof_branch_is_shared_across_all_referencing_contexts() {
    let branch = types::OrchestrationCacheVariableGetDataResponseOneOf0 {
        cache_variable_data: "Hello World!".to_string(),
        updated_at: None,
    };

    let global_put =
        types::UpdateExternalDataCacheVarDataOnGlobalOrchResponse::from(branch.clone());
    let service_put =
        types::UpdateExternalDataCacheVarDataOnServiceOrchResponse::from(branch.clone());
    let global_get = types::GetExternalDataCacheVarDataOnGlobalOrchResponse::from(branch.clone());
    let put_body = types::UpdateExternalDataCacheVarDataOnGlobalOrchBody::from(branch);

    // All four contexts serialize the shared branch to the same wire shape.
    let expected = serde_json::json!({"cache_variable_data": "Hello World!"});
    assert_eq!(serde_json::to_value(&global_put).unwrap(), expected);
    assert_eq!(serde_json::to_value(&service_put).unwrap(), expected);
    assert_eq!(serde_json::to_value(&global_get).unwrap(), expected);
    assert_eq!(serde_json::to_value(&put_body).unwrap(), expected);
}

/// `Tag` is `allOf: [<base reference object>, <tag-specific object>]`
/// where the base is the deep-pointer target hoisted to `TagAllOf0`. A
/// realistic payload mixing base fields (id, summary, self, html_url)
/// and tag fields (type, label — the spec's own example) must
/// deserialize with every field intact and round-trip losslessly.
#[test]
fn tag_round_trips_with_base_and_own_fields() {
    let payload = serde_json::json!({
        "id": "PT4KHLK",
        "type": "tag",
        "label": "Batman",
        "summary": "Batman",
        "self": "https://api.pagerduty.com/tags/PT4KHLK",
        "html_url": "https://subdomain.pagerduty.com/tags/PT4KHLK"
    });
    let tag: types::Tag = serde_json::from_value(payload.clone()).expect("Tag deserializes");

    assert_eq!(tag.id.as_deref(), Some("PT4KHLK"));
    assert_eq!(tag.type_, types::TagType::Tag);
    assert_eq!(tag.label.as_str(), "Batman");
    assert_eq!(tag.summary.as_deref(), Some("Batman"));
    assert_eq!(tag.self_.as_deref(), Some("https://api.pagerduty.com/tags/PT4KHLK"));
    assert_eq!(
        tag.html_url.as_deref(),
        Some("https://subdomain.pagerduty.com/tags/PT4KHLK")
    );

    // A serialization that silently drops a documented field must fail.
    let value = serde_json::to_value(&tag).expect("serializes");
    assert_eq!(value, payload);
}

/// `label` and `type` are required on the tag-specific half of the
/// allOf; the base half must stay optional. If the allOf merge with the
/// hoisted component got the requiredness wrong either way, this fails.
#[test]
fn tag_requires_label_and_type_but_not_base_fields() {
    let minimal: types::Tag = serde_json::from_value(serde_json::json!({
        "type": "tag",
        "label": "prod"
    }))
    .expect("base reference fields are all optional");
    assert_eq!(minimal.id, None);
    assert_eq!(minimal.summary, None);

    let missing_label =
        serde_json::from_value::<types::Tag>(serde_json::json!({"type": "tag"}));
    assert!(missing_label.is_err(), "label is required");

    let missing_type =
        serde_json::from_value::<types::Tag>(serde_json::json!({"label": "prod"}));
    assert!(missing_type.is_err(), "type is required");
}

/// The `type` enum documents exactly one value, `tag`.
#[test]
fn tag_type_rejects_undocumented_values() {
    assert_eq!("tag".parse::<types::TagType>().unwrap(), types::TagType::Tag);
    assert!("team".parse::<types::TagType>().is_err());
    assert!("Tag".parse::<types::TagType>().is_err());

    let wrong_type = serde_json::from_value::<types::Tag>(serde_json::json!({
        "type": "tag_reference",
        "label": "prod"
    }));
    assert!(wrong_type.is_err(), "Tag must reject undocumented type values");
}

/// `label` documents `maxLength: 191`; the generated guard must accept
/// the boundary and reject one past it.
#[test]
fn tag_label_enforces_documented_max_length() {
    let at_limit = "x".repeat(191);
    assert!(types::TagLabel::try_from(at_limit.as_str()).is_ok());

    let past_limit = "x".repeat(192);
    assert!(types::TagLabel::try_from(past_limit.as_str()).is_err());
    assert!(
        serde_json::from_value::<types::TagLabel>(serde_json::json!(past_limit)).is_err(),
        "over-long label must also be rejected on deserialize"
    );
}

/// The 33 deep-pointer sites all resolve to one hoisted component,
/// `TagAllOf0`. Pin that the named type exists and round-trips, and
/// that a second referencing context (`Reference`, whose allOf head is
/// the same pointer) carries the identical base fields merged in — the
/// observable effect of both sites resolving to the shared component.
#[test]
fn hoisted_tag_base_component_is_usable_from_both_contexts() {
    let base = serde_json::json!({
        "id": "PXPGF42",
        "summary": "A summary",
        "type": "service_reference",
        "self": "https://api.pagerduty.com/services/PXPGF42",
        "html_url": "https://subdomain.pagerduty.com/services/PXPGF42"
    });

    let hoisted: types::TagAllOf0 =
        serde_json::from_value(base.clone()).expect("hoisted component deserializes");
    assert_eq!(hoisted.id.as_deref(), Some("PXPGF42"));
    assert_eq!(hoisted.summary.as_deref(), Some("A summary"));
    assert_eq!(hoisted.type_.as_deref(), Some("service_reference"));
    assert_eq!(hoisted.self_.as_deref(), Some("https://api.pagerduty.com/services/PXPGF42"));
    assert_eq!(serde_json::to_value(&hoisted).expect("serializes"), base);

    // `Reference` = allOf [#/components/schemas/Tag/allOf/0, {id/type required}]:
    // the same payload must deserialize with the shared base fields intact.
    let reference: types::Reference =
        serde_json::from_value(base.clone()).expect("Reference deserializes");
    assert_eq!(reference.id, "PXPGF42");
    assert_eq!(reference.type_, "service_reference");
    assert_eq!(reference.summary.as_deref(), Some("A summary"));
    assert_eq!(
        reference.html_url.as_deref(),
        Some("https://subdomain.pagerduty.com/services/PXPGF42")
    );
    assert_eq!(serde_json::to_value(&reference).expect("serializes"), base);
}
