//! Generated Petstore 3.1 API client — conformance crate.
//!
//! Hermetic crate: spec is committed in this directory as petstore-31.yaml.
//! Used as a fast always-asserted PR-CI gate; the tiny spec ensures a quick
//! cold build and any regression in 3.1 baseline handling surfaces immediately.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let _client = Client::new("https://petstore.example.com/v1");
    }

    #[test]
    fn pet_round_trips_through_serde() {
        let pet: types::Pet =
            serde_json::from_str(r#"{"id": 7, "name": "rex", "tag": "dog"}"#)
                .expect("deserializes");
        assert_eq!(pet.id, 7);
        assert_eq!(pet.name, "rex");
        assert_eq!(pet.tag.as_deref(), Some("dog"));

        let value = serde_json::to_value(&pet).expect("serializes");
        assert_eq!(value["name"], "rex");
        assert_eq!(value["id"], 7);
        assert_eq!(value["tag"], "dog");
    }

    #[test]
    fn optional_fields_may_be_absent() {
        let pet: types::Pet =
            serde_json::from_str(r#"{"id": 1, "name": "min"}"#).expect("deserializes");
        assert_eq!(pet.tag, None);

        let value = serde_json::to_value(&pet).expect("serializes");
        assert!(value.get("tag").is_none(), "absent optional must stay off wire");
    }

    #[test]
    fn error_round_trips() {
        let error: types::Error =
            serde_json::from_str(r#"{"code": 404, "message": "not found"}"#)
                .expect("deserializes");
        assert_eq!(error.code, 404);
        assert_eq!(error.message, "not found");
    }
}

#[cfg(test)]
mod example_tests {
    include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
}
