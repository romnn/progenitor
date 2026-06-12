//! Generated Discord API client — conformance crate.
//!
//! Demonstrates using progenitor with the Discord API and pins, via
//! the tests below, properties of the generated code that a consumer
//! relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let client = Client::new("https://discord.com/api/v10");
        assert_eq!(client.baseurl(), "https://discord.com/api/v10");
    }

    #[test]
    fn message_create_request_round_trips() {
        // Creating a message is the central operation of this API; the
        // request must deserialize from the documented wire shape, with
        // optional fields generated as Options that stay off the wire
        // when unset.
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
        // Message components are Discord's discriminator-heavy oneOf: each
        // variant pins its `type` to a distinct integer const, so the
        // untagged enum must pick the variant from that discriminant alone
        // (10 = text display) and preserve it on re-serialization.
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
        // No component variant carries `type: 999`, so the const-validated
        // discriminant newtypes must reject the document outright instead
        // of falling through to some lenient variant.
        let result = serde_json::from_value::<types::MessageCreateRequestComponentsItem>(
            serde_json::json!({
                "type": 999,
                "content": "nope",
            }),
        );
        assert!(result.is_err());
    }
}
