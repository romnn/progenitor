//! Behavioral spot-checks against the client generated from the 3.1
//! petstore — the pattern for per-spec assertions: thin, focused on
//! constructs a past bug touched, checked by rustc and serde rather
//! than by matching generated source text.

use wild_petstore_31::types;

#[test]
fn pet_round_trips_through_serde() {
    let pet: types::Pet = serde_json::from_str(r#"{"id": 7, "name": "rex", "tag": "dog"}"#)
        .expect("deserializes");
    assert_eq!(pet.id, 7);
    assert_eq!(pet.name, "rex");
    // `tag` is optional in the spec and must be generated as an Option.
    assert_eq!(pet.tag.as_deref(), Some("dog"));

    let value = serde_json::to_value(&pet).expect("serializes");
    assert_eq!(value["name"], "rex");
}

#[test]
fn optional_fields_may_be_absent() {
    let pet: types::Pet =
        serde_json::from_str(r#"{"id": 1, "name": "min"}"#).expect("deserializes");
    assert_eq!(pet.tag, None);
}
