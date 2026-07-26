//! The buffered `Deserialize` form must be indistinguishable from the derive
//! on the wire.
//!
//! These are differential tests: the same schema is generated twice, once with
//! [`DeserializeImpl::Derived`] and once with [`DeserializeImpl::Buffered`],
//! and the emitted sources are compared structurally. The behavioural half —
//! that the two impls accept and reject byte-identical inputs, including
//! duplicate keys, absent-versus-null, and error text — lives in
//! `tests/de_runtime.rs`, which pulls in `src/de_runtime.rs` directly and so
//! exercises the very bytes that get emitted.

use schemars::schema::Schema;
use serde_json::json;
use typify_impl::{DeserializeImpl, TypeSpace, TypeSpaceSettings};

/// Emitted source with all whitespace removed.
///
/// `TokenStream::to_string` decides its own spacing, so matching on it directly
/// makes these tests fail for reasons that have nothing to do with the change
/// under test.
fn generate(schema: serde_json::Value, deserialize_impl: DeserializeImpl) -> String {
    let schema: Schema = serde_json::from_value(schema).expect("valid schema");
    let mut settings = TypeSpaceSettings::default();
    settings.with_deserialize_impl(deserialize_impl);
    let mut type_space = TypeSpace::new(&settings);
    type_space.add_type(&schema).expect("type generation");
    type_space
        .to_stream()
        .to_string()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// The marker for "this type got a buffered impl": its own `Build` impl.
/// Matching on `fn build` alone would also hit the runtime's trait declaration.
fn builds(out: &str, type_name: &str) -> bool {
    out.contains(&format!("self::de::Build<'de>for{type_name}"))
}

fn plain_struct() -> serde_json::Value {
    json!({
        "type": "object",
        "title": "Thing",
        "properties": {
            "name": { "type": "string" },
            "count": { "type": "integer" },
            "tag": { "type": "string" },
        },
        "required": ["name", "count"],
    })
}

#[test]
fn derived_is_the_default_and_is_unchanged() {
    let out = generate(plain_struct(), DeserializeImpl::Derived);
    assert!(
        out.contains("serde::Deserialize"),
        "the derived form must still derive Deserialize:\n{out}"
    );
    assert!(
        !builds(&out, "Thing"),
        "the derived form must not emit a builder:\n{out}"
    );
    assert!(
        !out.contains("pub(crate)modde"),
        "the derived form must not emit the runtime:\n{out}"
    );
}

#[test]
fn buffered_replaces_the_derive_with_a_two_part_impl() {
    let out = generate(plain_struct(), DeserializeImpl::Buffered);
    assert!(
        builds(&out, "Thing"),
        "expected a monomorphic builder:\n{out}"
    );
    assert!(
        out.contains("impl<'de>::serde::Deserialize<'de>forThing"),
        "expected a hand-written Deserialize impl:\n{out}"
    );
    assert!(
        out.contains("pub(crate)modde"),
        "expected the runtime module:\n{out}"
    );
}

#[test]
fn buffered_still_derives_serialize() {
    // Only `Deserialize` is replaced; the serialize half is cheap to derive and
    // has no format restriction, so it stays.
    let out = generate(plain_struct(), DeserializeImpl::Buffered);
    assert!(
        out.contains("Serialize"),
        "Serialize must still be derived:\n{out}"
    );
}

#[test]
fn buffered_uses_the_wire_name_not_the_rust_name() {
    // A property that has to be renamed to be a legal identifier must still be
    // looked up under its original key.
    let out = generate(
        json!({
            "type": "object",
            "title": "Thing",
            "properties": { "type": { "type": "string" } },
            "required": ["type"],
        }),
        DeserializeImpl::Buffered,
    );
    assert!(
        out.contains(r#""type""#),
        "the wire name must appear in the builder:\n{out}"
    );
}

#[test]
fn a_flattened_property_keeps_the_derive() {
    // `flatten` consumes the keys the struct does not name, which a
    // by-name lookup cannot express — so these must not be converted.
    let out = generate(
        json!({
            "type": "object",
            "title": "Thing",
            "properties": { "name": { "type": "string" } },
            "required": ["name"],
            "additionalProperties": { "type": "string" },
        }),
        DeserializeImpl::Buffered,
    );
    // Assert the precondition, or this test passes vacuously the day typify
    // stops lowering `additionalProperties` to a flattened map.
    assert!(
        out.contains("flatten"),
        "expected this schema to produce a flattened property:\n{out}"
    );
    assert!(
        !builds(&out, "Thing"),
        "a struct with a flattened property must keep the derive:\n{out}"
    );
}

#[test]
fn optional_properties_fall_back_to_a_default() {
    let out = generate(plain_struct(), DeserializeImpl::Buffered);
    assert!(
        out.contains("defaulted"),
        "an optional property should use the defaulting helper:\n{out}"
    );
    assert!(
        out.contains("required"),
        "a required property should use the required helper:\n{out}"
    );
}

/// Generate with the buffered impl requested, plus a caller customisation.
fn generate_customised(
    schema: serde_json::Value,
    customise: impl FnOnce(&mut TypeSpaceSettings),
) -> String {
    let schema: Schema = serde_json::from_value(schema).expect("valid schema");
    let mut settings = TypeSpaceSettings::default();
    settings.with_deserialize_impl(DeserializeImpl::Buffered);
    customise(&mut settings);
    let mut type_space = TypeSpace::new(&settings);
    type_space.add_type(&schema).expect("type generation");
    type_space
        .to_stream()
        .to_string()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

#[test]
fn an_injected_attribute_keeps_the_derive() {
    // A hand-written impl never reads container attributes, so honouring this
    // request and the buffered form at once is not possible. Silently ignoring
    // it would change the wire contract while still compiling — the worst
    // outcome — so the derive wins.
    let out = generate_customised(plain_struct(), |settings| {
        settings.with_attr("#[serde(rename_all = \"camelCase\")]".to_string());
    });
    assert!(
        out.contains("rename_all"),
        "the requested attribute must still be emitted:\n{out}"
    );
    assert!(
        !builds(&out, "Thing"),
        "a type carrying caller attributes must keep the derive:\n{out}"
    );
}

#[test]
fn a_patched_attribute_keeps_the_derive() {
    use typify_impl::TypeSpacePatch;

    let out = generate_customised(plain_struct(), |settings| {
        settings.with_patch(
            "Thing",
            TypeSpacePatch::default().with_attr("#[serde(deny_unknown_fields)]"),
        );
    });
    assert!(
        !builds(&out, "Thing"),
        "a patched type must keep the derive:\n{out}"
    );
}

#[test]
fn an_injected_deserialize_derive_keeps_the_derive() {
    // Emitting both would be two `Deserialize` impls for one type. That at
    // least fails to compile rather than misbehaving, but there is no reason
    // to make the caller discover it.
    for spelling in ["Deserialize", "serde::Deserialize", "::serde::Deserialize"] {
        let out = generate_customised(plain_struct(), |settings| {
            settings.with_derive(spelling.to_string());
        });
        assert!(
            !builds(&out, "Thing"),
            "`{spelling}` must suppress the buffered impl:\n{out}"
        );
    }
}

#[test]
fn an_unrelated_derive_still_allows_the_buffered_impl() {
    // The gate must not be so blunt that the common customisation loses the
    // optimisation.
    let out = generate_customised(plain_struct(), |settings| {
        settings.with_derive("schemars::JsonSchema".to_string());
    });
    assert!(
        builds(&out, "Thing"),
        "an unrelated derive must not disqualify the type:\n{out}"
    );
}
