//! Generated Discord API client — conformance crate.
//!
//! Tests cover:
//! - Message creation and component union discriminants (original suite)
//! - Inline enum discriminant guards merged with allOf $refs (wild suite):
//!   ChannelSelectDefaultValue, RoleSelectDefaultValue, integration types,
//!   OAuth2 scopes items, embedded activity location kinds, and form fields.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    // Original conformance suite

    #[test]
    fn client_constructs() {
        let client = Client::new("https://discord.com/api/v10");
        assert_eq!(client.baseurl(), "https://discord.com/api/v10");
    }

    #[test]
    fn message_create_request_round_trips() {
        let request: types::MessageCreateRequest = serde_json::from_value(serde_json::json!({
            "content": "hello world",
            "tts": false,
        }))
        .expect("documented request shape deserializes");
        let value = serde_json::to_value(&request).expect("serializes");
        assert_eq!(value["content"], "hello world");
        assert!(
            value.get("flags").is_none(),
            "unset optional fields must not serialize"
        );
    }

    #[test]
    fn component_union_discriminates_on_type() {
        let component: types::MessageCreateRequestComponentsItem =
            serde_json::from_value(serde_json::json!({
                "type": 10,
                "content": "**bold**",
            }))
            .expect("text display component deserializes");
        assert!(matches!(
            component,
            types::MessageCreateRequestComponentsItem::TextDisplayComponentForMessageRequest(_)
        ));
        let value = serde_json::to_value(&component).expect("serializes");
        assert_eq!(value["type"], 10);
    }

    #[test]
    fn component_union_rejects_unknown_discriminant() {
        let result = serde_json::from_value::<types::MessageCreateRequestComponentsItem>(
            serde_json::json!({ "type": 999, "content": "nope" }),
        );
        assert!(result.is_err());
    }

    // Wild suite: inline enum discriminant guards

    #[test]
    fn channel_select_default_value_accepts_channel_discriminant() {
        let payload = serde_json::json!({"id": "123", "type": "channel"});
        let value: types::ChannelSelectDefaultValue =
            serde_json::from_value(payload.clone()).expect("documented discriminant deserializes");
        assert_eq!(value.id.as_str(), "123");
        assert_eq!(
            *value.type_,
            types::SnowflakeSelectDefaultValueTypes::Channel
        );
        assert_eq!(
            serde_json::to_value(&value).expect("serializes"),
            payload
        );
    }

    #[test]
    fn channel_select_default_value_rejects_shared_enum_value_outside_inline_subset() {
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
        for ok in ["\"twitch\"", "\"youtube\""] {
            let parsed: types::ExternalConnectionIntegrationResponseType =
                serde_json::from_str(ok).expect("documented integration kind deserializes");
            assert_eq!(serde_json::to_string(&parsed).unwrap(), ok);
        }
        for bad in ["\"discord\"", "\"guild_subscription\"", "\"bogus\""] {
            assert!(
                serde_json::from_str::<types::ExternalConnectionIntegrationResponseType>(bad)
                    .is_err(),
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

        let err = serde_json::from_value::<types::ApplicationOAuth2InstallParams>(
            serde_json::json!({"scopes": ["identify"]}),
        );
        assert!(err.is_err(), "'identify' must be rejected by the scopes item guard");
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
}
