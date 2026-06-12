//! Generated Figma REST API client — conformance crate.
//!
//! Demonstrates using progenitor with the Figma REST API (OpenAPI 3.1,
//! YAML, discriminated unions on node types) and pins, via the tests
//! below, properties of the generated code that a consumer relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let client = Client::new("https://api.figma.com");
        assert_eq!(client.baseurl(), "https://api.figma.com");
    }

    #[test]
    fn get_file_response_round_trips() {
        // GET /v1/files/{file_key} is the central operation; its response
        // must deserialize from the documented wire shape (camelCase keys
        // via serde renames) with optionals that stay off the wire when
        // unset.
        let response: types::GetFileResponse = serde_json::from_value(serde_json::json!({
            "name": "Design System",
            "role": "owner",
            "lastModified": "2026-01-01T00:00:00Z",
            "editorType": "figma",
            "version": "12345",
            "schemaVersion": 0,
            "components": {},
            "componentSets": {},
            "styles": {},
            "document": {
                "id": "0:0",
                "name": "Document",
                "type": "DOCUMENT",
                "scrollBehavior": "SCROLLS",
                "children": [],
            },
        }))
        .expect("documented file response deserializes");
        assert_eq!(response.name, "Design System");
        assert_eq!(response.document.id, "0:0");

        let value = serde_json::to_value(&response).expect("serializes");
        assert_eq!(value["editorType"], "figma");
        assert!(
            value.get("thumbnailUrl").is_none(),
            "unset optional fields must not serialize"
        );
        assert!(
            value.get("branches").is_none(),
            "empty defaulted collections must not serialize"
        );
    }

    #[test]
    fn node_union_parses_discriminated_variant() {
        // Node is the spec's discriminated union (propertyName "type");
        // each variant's `type` field is a single-value enum, so the
        // wire-level discriminator must select exactly one variant.
        let node: types::Node = serde_json::from_value(serde_json::json!({
            "id": "1:2",
            "name": "Spotify embed",
            "type": "EMBED",
            "scrollBehavior": "SCROLLS",
        }))
        .expect("node with EMBED discriminator deserializes");
        let types::Node::EmbedNode(embed) = node else {
            panic!("type=EMBED must select the EmbedNode variant");
        };
        assert_eq!(embed.id, "1:2");
        // `visible` is schema-defaulted to true when absent.
        assert!(embed.visible);

        let value = serde_json::to_value(&embed).expect("serializes");
        assert_eq!(value["type"], "EMBED", "discriminator survives round trip");
    }
}
