//! Generated Qdrant API client — conformance crate.
//!
//! Demonstrates using progenitor with the Qdrant vector-database API and
//! pins, via the tests below, properties of the generated code that a
//! consumer relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let client = Client::new("http://localhost:6333");
        assert_eq!(client.baseurl(), "http://localhost:6333");
    }

    #[test]
    fn point_struct_round_trips() {
        // PointStruct is the unit of every upsert; named-vector form.
        let point: types::PointStruct = serde_json::from_value(serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "vector": {"image": [0.1, 0.2, 0.3]},
        }))
        .expect("documented point shape deserializes");
        assert!(matches!(point.id, types::ExtendedPointId::Uuid(_)));
        let value = serde_json::to_value(&point).expect("serializes");
        assert!(
            value.get("payload").is_none(),
            "unset optional payload must not serialize"
        );
    }

    #[test]
    fn create_collection_round_trips() {
        let request: types::CreateCollection = serde_json::from_value(serde_json::json!({
            "vectors": {"size": 768, "distance": "Cosine"},
        }))
        .expect("documented create-collection shape deserializes");
        let vectors = request.vectors.as_ref().expect("vectors set");
        assert_eq!(
            vectors
                .subtype_0
                .as_ref()
                .expect("single-vector params")
                .distance,
            types::Distance::Cosine
        );
        let value = serde_json::to_value(&request).expect("serializes");
        assert!(
            value.get("shard_number").is_none(),
            "unset optional fields must not serialize"
        );
    }

    #[test]
    fn extended_point_id_accepts_both_wire_forms() {
        let numeric: types::ExtendedPointId =
            serde_json::from_value(serde_json::json!(42)).expect("integer id parses");
        assert!(matches!(numeric, types::ExtendedPointId::Uint64(42)));
        let uuid: types::ExtendedPointId =
            serde_json::from_value(serde_json::json!("550e8400-e29b-41d4-a716-446655440000"))
                .expect("uuid id parses");
        assert!(matches!(uuid, types::ExtendedPointId::Uuid(_)));
    }
}
