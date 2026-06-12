//! Behavioral spot-checks against the client generated from the Discord
//! spec, which historically failed generation on inline enum
//! discriminants merged with allOf $refs to shared enum components,
//! e.g. `ChannelSelectDefaultValue.type` is `{"enum": ["channel"]}`
//! plus `allOf: [$ref SnowflakeSelectDefaultValueTypes]`. The generator
//! now emits a guard newtype around the shared enum whose
//! TryFrom/Deserialize restricts membership to the inline subset; these
//! tests pin that the guard accepts exactly the documented values and
//! rejects both undocumented strings and shared-enum values outside the
//! inline subset.

use wild_discord::types;

#[test]
fn channel_select_default_value_accepts_channel_discriminant() {
    let payload = serde_json::json!({"id": "123", "type": "channel"});
    let value: types::ChannelSelectDefaultValue =
        serde_json::from_value(payload.clone()).expect("documented discriminant deserializes");

    assert_eq!(value.id.as_str(), "123");
    // The guard newtype derefs to the shared enum component.
    assert_eq!(
        *value.type_,
        types::SnowflakeSelectDefaultValueTypes::Channel
    );

    // Both documented fields must survive a round-trip unchanged.
    let round_tripped = serde_json::to_value(&value).expect("serializes");
    assert_eq!(round_tripped, payload);
}

#[test]
fn channel_select_default_value_rejects_shared_enum_value_outside_inline_subset() {
    // "role" is a valid SnowflakeSelectDefaultValueTypes member, but the
    // inline enum on ChannelSelectDefaultValue.type only allows "channel";
    // the TryFrom guard must reject it at deserialization.
    let err = serde_json::from_value::<types::ChannelSelectDefaultValue>(
        serde_json::json!({"id": "123", "type": "role"}),
    );
    assert!(err.is_err(), "'role' must not satisfy a channel-only discriminant");

    let err = serde_json::from_value::<types::ChannelSelectDefaultValue>(
        serde_json::json!({"id": "123", "type": "user"}),
    );
    assert!(err.is_err(), "'user' must not satisfy a channel-only discriminant");
}

#[test]
fn channel_select_default_value_rejects_undocumented_discriminant() {
    let err = serde_json::from_value::<types::ChannelSelectDefaultValue>(
        serde_json::json!({"id": "123", "type": "bogus"}),
    );
    assert!(err.is_err(), "undocumented discriminant must be rejected");
}

#[test]
fn role_select_default_value_guard_mirrors_channel_guard() {
    let payload = serde_json::json!({"id": "9", "type": "role"});
    let value: types::RoleSelectDefaultValue =
        serde_json::from_value(payload.clone()).expect("'role' deserializes");
    assert_eq!(*value.type_, types::SnowflakeSelectDefaultValueTypes::Role);
    assert_eq!(serde_json::to_value(&value).expect("serializes"), payload);

    let err = serde_json::from_value::<types::RoleSelectDefaultValue>(
        serde_json::json!({"id": "9", "type": "channel"}),
    );
    assert!(err.is_err(), "'channel' must not satisfy a role-only discriminant");
}

#[test]
fn select_default_value_try_from_guard_enforces_inline_subset() {
    use types::SnowflakeSelectDefaultValueTypes as Shared;

    assert!(types::ChannelSelectDefaultValueType::try_from(Shared::Channel).is_ok());
    assert!(types::ChannelSelectDefaultValueType::try_from(Shared::Role).is_err());
    assert!(types::ChannelSelectDefaultValueType::try_from(Shared::User).is_err());

    assert!(types::RoleSelectDefaultValueType::try_from(Shared::Role).is_ok());
    assert!(types::RoleSelectDefaultValueType::try_from(Shared::Channel).is_err());
}

#[test]
fn integration_type_guard_allows_only_the_two_external_kinds() {
    // ExternalConnectionIntegrationResponse.type inlines ["twitch",
    // "youtube"] over the four-member IntegrationTypes component.
    for ok in ["\"twitch\"", "\"youtube\""] {
        let parsed: types::ExternalConnectionIntegrationResponseType =
            serde_json::from_str(ok).expect("documented integration kind deserializes");
        assert_eq!(serde_json::to_string(&parsed).unwrap(), ok);
    }
    for bad in ["\"discord\"", "\"guild_subscription\"", "\"bogus\""] {
        assert!(
            serde_json::from_str::<types::ExternalConnectionIntegrationResponseType>(bad).is_err(),
            "{bad} must be rejected by the external-connection guard"
        );
    }
}

#[test]
fn oauth2_scopes_items_guard_restricts_install_params() {
    let payload = serde_json::json!({"scopes": ["applications.commands", "bot"]});
    let params: types::ApplicationOAuth2InstallParams =
        serde_json::from_value(payload.clone()).expect("documented scopes deserialize");
    let scopes = params.scopes.as_ref().expect("scopes field survives");
    assert_eq!(scopes.len(), 2);
    assert_eq!(*scopes[0], types::OAuth2Scopes::ApplicationsCommands);
    assert_eq!(*scopes[1], types::OAuth2Scopes::Bot);
    assert_eq!(serde_json::to_value(&params).expect("serializes"), payload);

    // "identify" is a valid OAuth2Scopes member, but the install-params
    // items enum only allows applications.commands and bot.
    let err = serde_json::from_value::<types::ApplicationOAuth2InstallParams>(
        serde_json::json!({"scopes": ["identify"]}),
    );
    assert!(err.is_err(), "'identify' must be rejected by the scopes item guard");
}

#[test]
fn embedded_activity_location_kind_guard() {
    let payload = serde_json::json!({
        "id": "loc",
        "kind": "gc",
        "channel_id": "42",
        "guild_id": "7"
    });
    let location: types::GuildChannelLocation =
        serde_json::from_value(payload.clone()).expect("'gc' deserializes");
    assert_eq!(*location.kind, types::EmbeddedActivityLocationKind::Gc);
    assert_eq!(location.channel_id.as_str(), "42");
    assert_eq!(location.guild_id.as_str(), "7");
    assert_eq!(serde_json::to_value(&location).expect("serializes"), payload);

    // "pc" and "party" are valid EmbeddedActivityLocationKind members but
    // not documented for guild channel locations.
    for bad in ["pc", "party"] {
        let err = serde_json::from_value::<types::GuildChannelLocation>(serde_json::json!({
            "id": "loc",
            "kind": bad,
            "channel_id": "42",
            "guild_id": "7"
        }));
        assert!(err.is_err(), "'{bad}' must be rejected for guild channel locations");
    }
}

#[test]
fn verification_form_field_type_guard() {
    let payload = serde_json::json!({"field_type": "TERMS", "values": ["rule one"]});
    let field: types::TermsFormFieldResponse =
        serde_json::from_value(payload.clone()).expect("'TERMS' deserializes");
    assert_eq!(
        *field.field_type,
        types::GuildMemberVerificationFormFieldType::Terms
    );
    assert_eq!(field.values, vec!["rule one".to_string()]);
    assert_eq!(serde_json::to_value(&field).expect("serializes"), payload);

    // TEXT_INPUT is a valid GuildMemberVerificationFormFieldType member
    // but not documented for the TERMS response variant.
    let err = serde_json::from_value::<types::TermsFormFieldResponse>(
        serde_json::json!({"field_type": "TEXT_INPUT", "values": []}),
    );
    assert!(err.is_err(), "'TEXT_INPUT' must be rejected for a terms field");
}

#[test]
fn event_webhooks_action_types_guard() {
    let parsed: types::ApplicationFormPartialEventWebhooksTypesItem =
        serde_json::from_str("\"APPLICATION_AUTHORIZED\"").expect("documented event deserializes");
    assert_eq!(*parsed, types::ActionTypes::ApplicationAuthorized);

    // TYPING_START is a valid ActionTypes member, but the inline enum for
    // event_webhooks_types only allows the webhook-event subset.
    assert!(
        serde_json::from_str::<types::ApplicationFormPartialEventWebhooksTypesItem>(
            "\"TYPING_START\""
        )
        .is_err(),
        "'TYPING_START' must be rejected by the event-webhooks item guard"
    );
}

#[test]
fn message_create_request_round_trips_content_and_tts() {
    let payload = serde_json::json!({"content": "hello there", "tts": true});
    let request: types::MessageCreateRequest =
        serde_json::from_value(payload.clone()).expect("deserializes");
    assert_eq!(
        request.content.as_ref().expect("content survives").as_str(),
        "hello there"
    );
    assert_eq!(request.tts, Some(true));

    // Unset optional fields must be skipped, so the round-trip is exact.
    assert_eq!(serde_json::to_value(&request).expect("serializes"), payload);
}

#[test]
fn message_create_request_content_length_guard() {
    let max = "x".repeat(4000);
    let ok = serde_json::json!({ "content": max });
    assert!(
        serde_json::from_value::<types::MessageCreateRequest>(ok).is_ok(),
        "4000-char content is documented as valid"
    );

    let too_long = "x".repeat(4001);
    let err = serde_json::from_value::<types::MessageCreateRequest>(
        serde_json::json!({ "content": too_long }),
    );
    assert!(err.is_err(), "content over 4000 chars must be rejected");
}
