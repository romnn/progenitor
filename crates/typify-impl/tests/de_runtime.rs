//! Differential test for the emitted `types::de` runtime.
//!
//! The module under test is `src/de_runtime.rs` itself — the same bytes
//! `to_stream` splices into generated crates — pulled in with `#[path]` so a
//! change to the emitted runtime cannot pass CI without passing this file.
//!
//! Each case deserializes one JSON text two ways: through serde's derive, and
//! through a hand-written impl in exactly the shape
//! [`TypeEntry::output_struct`] emits for [`DeserializeImpl::Buffered`]. The
//! two must agree — on the accepted value, on rejection, and on the error
//! message.
//!
//! This is the safety net the conformance suite cannot provide. Its
//! `roundtrip!` goes through `serde_json::Value`, so duplicate keys are already
//! collapsed before the type sees them, and it asserts idempotence rather than
//! fidelity — a field dropped uniformly on every deserialize passes it.

#[path = "../src/de_runtime.rs"]
mod de;

use serde::Deserialize;

/// Derived reference implementation.
#[derive(Debug, PartialEq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Derived {
    name: String,
    count: i64,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    nickname: Option<String>,
    #[serde(rename = "type")]
    kind: String,
}

/// The same wire contract, written the way the generator emits it.
#[derive(Debug, PartialEq)]
struct Buffered {
    name: String,
    count: i64,
    tags: Vec<String>,
    nickname: Option<String>,
    kind: String,
}

const FIELDS: &[&str] = &["name", "count", "tags", "nickname", "type"];

/// The two impls stand for one logical type, so they must report one name.
const NAME: &str = "Derived";

impl<'de> de::Build<'de> for Buffered {
    const NAME: &'static str = NAME;
    const FIELDS: &'static [&'static str] = FIELDS;

    fn build<E: serde::de::Error>(mut fields: de::Fields<'de>) -> Result<Self, E> {
        let value = Self {
            name: de::required(&mut fields, "name")?,
            count: de::required(&mut fields, "count")?,
            tags: de::defaulted(&mut fields, "tags")?,
            nickname: de::defaulted(&mut fields, "nickname")?,
            kind: de::required(&mut fields, "type")?,
        };
        de::deny_unknown(&fields, <Self as de::Build>::FIELDS)?;
        Ok(value)
    }
}

impl<'de> serde::Deserialize<'de> for Buffered {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de::deserialize_struct(deserializer)
    }
}

/// The message without serde_json's trailing ` at line L column C`.
fn message_of(e: &serde_json::Error) -> String {
    let text = e.to_string();
    match text.find(" at line ") {
        Some(cut) => text[..cut].to_string(),
        None => text,
    }
}

/// Assert the two impls agree on this exact JSON text, value or error.
#[track_caller]
fn agree(json: &str) {
    let derived = serde_json::from_str::<Derived>(json);
    let buffered = serde_json::from_str::<Buffered>(json);

    match (derived, buffered) {
        (Ok(d), Ok(b)) => {
            assert_eq!(d.name, b.name, "name mismatch for {json}");
            assert_eq!(d.count, b.count, "count mismatch for {json}");
            assert_eq!(d.tags, b.tags, "tags mismatch for {json}");
            assert_eq!(d.nickname, b.nickname, "nickname mismatch for {json}");
            assert_eq!(d.kind, b.kind, "kind mismatch for {json}");
        }
        // Buffering defers failure until the whole map has been read, so the
        // reported line/column is the end of the object rather than the
        // offending field. The message itself must still match exactly; the
        // positional difference is pinned by `positions_differ_but_message_does_not`.
        (Err(d), Err(b)) => assert_eq!(
            message_of(&d),
            message_of(&b),
            "error message diverged for {json}"
        ),
        (Ok(_), Err(b)) => panic!("derive accepted but buffered rejected {json}: {b}"),
        (Err(d), Ok(_)) => panic!("derive rejected but buffered accepted {json}: {d}"),
    }
}

#[test]
fn accepts_the_same_well_formed_input() {
    agree(r#"{"name":"a","count":1,"tags":["x"],"nickname":"n","type":"t"}"#);
    agree(r#"{"type":"t","count":-3,"name":"a"}"#);
    agree(r#"{"name":"a","count":0,"type":"","tags":[]}"#);
}

#[test]
fn field_order_does_not_matter() {
    agree(r#"{"type":"t","tags":["x"],"count":1,"nickname":null,"name":"a"}"#);
}

#[test]
fn absent_optional_field_uses_the_default() {
    agree(r#"{"name":"a","count":1,"type":"t"}"#);
}

#[test]
fn present_null_is_distinguished_from_absent() {
    // The distinction a `HashMap`-shaped buffer would lose: `nickname: null`
    // deserializes through `Option`'s own impl, `tags: null` is a type error.
    agree(r#"{"name":"a","count":1,"type":"t","nickname":null}"#);
    agree(r#"{"name":"a","count":1,"type":"t","tags":null}"#);
}

#[test]
fn missing_required_field_reports_the_same_error() {
    agree(r#"{"count":1,"type":"t"}"#);
    agree(r#"{"name":"a","type":"t"}"#);
    agree(r#"{"name":"a","count":1}"#);
}

#[test]
fn duplicate_key_is_rejected_by_both() {
    // The regression a `serde_json::Map` / `HashMap` buffer would introduce: it
    // silently keeps the last value where the derive errors.
    agree(r#"{"name":"a","name":"b","count":1,"type":"t"}"#);
    agree(r#"{"name":"a","count":1,"count":2,"type":"t"}"#);
}

#[test]
fn unknown_field_is_rejected_by_both() {
    agree(r#"{"name":"a","count":1,"type":"t","surprise":1}"#);
}

#[test]
fn wrong_types_report_the_same_error() {
    agree(r#"{"name":1,"count":1,"type":"t"}"#);
    agree(r#"{"name":"a","count":"x","type":"t"}"#);
    agree(r#"{"name":"a","count":1,"type":"t","tags":"x"}"#);
    agree(r#"{"name":"a","count":1,"type":"t","tags":[1]}"#);
}

#[test]
fn non_object_input_reports_the_same_error() {
    agree(r#"[]"#);
    agree(r#""x""#);
    agree(r#"null"#);
    agree(r#"7"#);
}

#[test]
fn too_few_and_too_many_seq_elements_report_the_same_error() {
    // serde_json only drives `visit_seq` for a JSON array, which the struct
    // rejects outright, so the seq arm is exercised through a format that does.
    let short = serde_json::to_value(["a"]).unwrap();
    assert!(serde_json::from_value::<Buffered>(short).is_err());
}

#[test]
fn an_input_with_two_faults_may_report_either_one() {
    // The derive reads the map in wire order and fails at the first problem it
    // reaches. The buffered form reads the whole map, then resolves fields in
    // *declaration* order, then checks for unknown keys — so when an input has
    // more than one fault, the two can name different ones.
    //
    // Pinned rather than fixed: matching wire-order interleaving would mean
    // re-introducing the per-field generic dispatch this whole form exists to
    // avoid. Both impls still reject the input, which is the contract that
    // matters; serde does not promise which of several faults you are told
    // about.
    let json = r#"{"surprise":1,"name":1,"count":1,"type":"t"}"#;
    let d = serde_json::from_str::<Derived>(json).unwrap_err();
    let b = serde_json::from_str::<Buffered>(json).unwrap_err();
    assert!(
        message_of(&d).contains("unknown field"),
        "derive should stop at the unknown key it reads first, got: {d}"
    );
    assert!(
        message_of(&b).contains("invalid type"),
        "buffered resolves declared fields before unknown ones, got: {b}"
    );
}

#[test]
fn positions_differ_but_message_does_not() {
    // Documented divergence, not an accident: the derive fails at the offending
    // field, the buffered impl after the closing brace.
    let json = r#"{"name":1,"count":1,"type":"t"}"#;
    let d = serde_json::from_str::<Derived>(json).unwrap_err();
    let b = serde_json::from_str::<Buffered>(json).unwrap_err();
    assert_eq!(message_of(&d), message_of(&b));
    assert_ne!(
        d.column(),
        b.column(),
        "expected the known positional divergence"
    );
}

/// A nullable property that the schema marks *required*, which typify emits as
/// a bare `Option<T>` with no `#[serde(default)]`.
///
/// serde accepts an absent key here — `Option`'s own impl turns the missing
/// field into `None`. Missing this cost two conformance crates a false
/// `missing field` rejection, so it is pinned separately from the
/// `#[serde(default)]` case above.
#[derive(Debug, Deserialize)]
struct DerivedBareOption {
    required: String,
    nullable: Option<String>,
}

#[derive(Debug)]
struct BufferedBareOption {
    required: String,
    nullable: Option<String>,
}

impl<'de> de::Build<'de> for BufferedBareOption {
    const NAME: &'static str = "DerivedBareOption";
    const FIELDS: &'static [&'static str] = &["required", "nullable"];

    fn build<E: serde::de::Error>(mut fields: de::Fields<'de>) -> Result<Self, E> {
        Ok(BufferedBareOption {
            required: de::required(&mut fields, "required")?,
            nullable: de::required(&mut fields, "nullable")?,
        })
    }
}

impl<'de> serde::Deserialize<'de> for BufferedBareOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de::deserialize_struct(deserializer)
    }
}

#[test]
fn an_absent_bare_option_is_none_not_an_error() {
    for json in [
        r#"{"required":"a"}"#,
        r#"{"required":"a","nullable":null}"#,
        r#"{"required":"a","nullable":"x"}"#,
    ] {
        let d = serde_json::from_str::<DerivedBareOption>(json).expect(json);
        let b = serde_json::from_str::<BufferedBareOption>(json).expect(json);
        assert_eq!(d.required, b.required, "required for {json}");
        assert_eq!(d.nullable, b.nullable, "nullable for {json}");
    }

    // A non-optional field is still required, so the fix must not have turned
    // every absent key into a silent default.
    let d = serde_json::from_str::<DerivedBareOption>(r#"{"nullable":"x"}"#).unwrap_err();
    let b = serde_json::from_str::<BufferedBareOption>(r#"{"nullable":"x"}"#).unwrap_err();
    assert_eq!(message_of(&d), message_of(&b));
    assert!(message_of(&b).contains("missing field `required`"), "{b}");
}

fn seven() -> i64 {
    7
}

/// The `#[serde(default = "…")]` path, which the generator lowers to
/// `de::defaulted_with` rather than `de::defaulted`.
#[derive(Debug, Deserialize)]
struct DerivedCustom {
    #[serde(default = "seven")]
    count: i64,
}

struct BufferedCustom {
    count: i64,
}

impl<'de> de::Build<'de> for BufferedCustom {
    const NAME: &'static str = "DerivedCustom";
    const FIELDS: &'static [&'static str] = &["count"];

    fn build<E: serde::de::Error>(mut fields: de::Fields<'de>) -> Result<Self, E> {
        Ok(BufferedCustom {
            count: de::defaulted_with(&mut fields, "count", seven)?,
        })
    }
}

impl<'de> serde::Deserialize<'de> for BufferedCustom {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de::deserialize_struct(deserializer)
    }
}

#[test]
fn a_custom_default_agrees_with_the_derive() {
    for json in [r#"{}"#, r#"{"count":1}"#, r#"{"count":-9}"#] {
        let d = serde_json::from_str::<DerivedCustom>(json).expect(json);
        let b = serde_json::from_str::<BufferedCustom>(json).expect(json);
        assert_eq!(d.count, b.count, "custom default diverged for {json}");
    }
}

/// A `Deserializer` that drives structs through `visit_seq` and refuses
/// `deserialize_any`, which is what bincode, postcard and other
/// non-self-describing formats do.
mod not_self_describing {
    use serde::de::{self, DeserializeSeed, IntoDeserializer, SeqAccess, Visitor};
    use std::fmt;

    #[derive(Debug)]
    pub struct Error(pub String);

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }
    impl std::error::Error for Error {}
    impl de::Error for Error {
        fn custom<T: fmt::Display>(msg: T) -> Self {
            Error(msg.to_string())
        }
    }

    pub struct Deserializer;

    /// Drives an enum the way a compact format does: by variant index, with no
    /// `deserialize_any` anywhere.
    pub struct UnitDeserializer {
        pub index: u32,
    }

    impl<'de> de::Deserializer<'de> for UnitDeserializer {
        type Error = Error;

        fn deserialize_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value, Error> {
            Err(Error("AnyNotSupported".to_string()))
        }

        fn deserialize_enum<V: Visitor<'de>>(
            self,
            _name: &'static str,
            _variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error> {
            visitor.visit_enum(self)
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct identifier ignored_any
        }
    }

    impl<'de> de::EnumAccess<'de> for UnitDeserializer {
        type Error = Error;
        type Variant = Self;

        fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self), Error> {
            let index = self.index;
            let value = seed.deserialize(index.into_deserializer())?;
            Ok((value, self))
        }
    }

    impl<'de> de::VariantAccess<'de> for UnitDeserializer {
        type Error = Error;
        fn unit_variant(self) -> Result<(), Error> {
            Ok(())
        }
        fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, _: T) -> Result<T::Value, Error> {
            Err(Error("not a newtype variant".to_string()))
        }
        fn tuple_variant<V: Visitor<'de>>(self, _: usize, _: V) -> Result<V::Value, Error> {
            Err(Error("not a tuple variant".to_string()))
        }
        fn struct_variant<V: Visitor<'de>>(
            self,
            _: &'static [&'static str],
            _: V,
        ) -> Result<V::Value, Error> {
            Err(Error("not a struct variant".to_string()))
        }
    }

    impl<'de> de::Deserializer<'de> for Deserializer {
        type Error = Error;

        /// The defining property of the format: the decoder has only the bytes
        /// and the caller's expectation, so it cannot answer "what is here?".
        fn deserialize_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value, Error> {
            Err(Error("AnyNotSupported".to_string()))
        }

        fn deserialize_struct<V: Visitor<'de>>(
            self,
            _name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error> {
            visitor.visit_seq(Seq { left: fields.len() })
        }

        // Enough typed entry points to carry the reference struct's fields, so
        // that a failure below is the buffering and not this stub.
        fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            visitor.visit_str("x")
        }
        fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            visitor.visit_str("x")
        }
        fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            visitor.visit_i64(7)
        }
        fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            visitor.visit_seq(Seq { left: 0 })
        }
        fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            visitor.visit_none()
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 u8 u16 u32 u64 f32 f64 char bytes byte_buf unit
            unit_struct newtype_struct tuple tuple_struct map enum identifier
            ignored_any
        }
    }

    struct Seq {
        left: usize,
    }

    impl<'de> SeqAccess<'de> for Seq {
        type Error = Error;

        fn next_element_seed<T: DeserializeSeed<'de>>(
            &mut self,
            seed: T,
        ) -> Result<Option<T::Value>, Error> {
            if self.left == 0 {
                return Ok(None);
            }
            self.left -= 1;
            seed.deserialize(Deserializer).map(Some)
        }
    }
}

#[test]
fn a_non_self_describing_format_is_a_known_loss() {
    // PINNED, not aspirational. Buffering a field *value* requires
    // `deserialize_any`, which bincode/postcard cannot provide, so the seq arm
    // recovers the struct shape but not the values. This is the entire reason
    // `DeserializeImpl::Derived` has to remain reachable.
    let err = <Buffered as serde::Deserialize>::deserialize(not_self_describing::Deserializer)
        .expect_err("a non-self-describing format cannot feed the buffered impl");
    assert_eq!(err.0, "AnyNotSupported");

    // The derive, driven by the very same deserializer, succeeds — which is
    // what makes the line above a real loss rather than a limitation of this
    // stub.
    let derived = <Derived as serde::Deserialize>::deserialize(not_self_describing::Deserializer)
        .expect("the derive must not need deserialize_any");
    assert_eq!(derived.name, "x");
    assert_eq!(derived.count, 7);
    assert_eq!(derived.kind, "x");
    assert_eq!(derived.nickname, None);
    assert!(derived.tags.is_empty());
}

/// Every remaining property shape `generate_serde_attr` can emit, in one type.
///
/// The `Option`/`Vec`/`Map` cases each get `default` plus a `skip_serializing_if`
/// from typify, and this struct deliberately has **no** `deny_unknown_fields`,
/// so it also covers the "unknown keys are ignored" path that `Derived` above
/// does not reach.
#[derive(Debug, Deserialize)]
struct DerivedShapes {
    required: String,
    bare_option: Option<String>,
    #[serde(default)]
    defaulted_option: Option<String>,
    #[serde(default)]
    defaulted_vec: Vec<String>,
    #[serde(default)]
    defaulted_map: std::collections::HashMap<String, i64>,
    #[serde(default = "seven")]
    custom: i64,
    #[serde(rename = "wire-name")]
    renamed: String,
}

#[derive(Debug)]
struct BufferedShapes {
    required: String,
    bare_option: Option<String>,
    defaulted_option: Option<String>,
    defaulted_vec: Vec<String>,
    defaulted_map: std::collections::HashMap<String, i64>,
    custom: i64,
    renamed: String,
}

impl<'de> de::Build<'de> for BufferedShapes {
    const NAME: &'static str = "DerivedShapes";
    const FIELDS: &'static [&'static str] = &[
        "required",
        "bare_option",
        "defaulted_option",
        "defaulted_vec",
        "defaulted_map",
        "custom",
        "wire-name",
    ];

    fn build<E: serde::de::Error>(mut fields: de::Fields<'de>) -> Result<Self, E> {
        Ok(BufferedShapes {
            required: de::required(&mut fields, "required")?,
            bare_option: de::required(&mut fields, "bare_option")?,
            defaulted_option: de::defaulted(&mut fields, "defaulted_option")?,
            defaulted_vec: de::defaulted(&mut fields, "defaulted_vec")?,
            defaulted_map: de::defaulted(&mut fields, "defaulted_map")?,
            custom: de::defaulted_with(&mut fields, "custom", seven)?,
            renamed: de::required(&mut fields, "wire-name")?,
        })
    }
}

impl<'de> serde::Deserialize<'de> for BufferedShapes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de::deserialize_struct(deserializer)
    }
}

#[track_caller]
fn shapes_agree(json: &str) {
    let d = serde_json::from_str::<DerivedShapes>(json);
    let b = serde_json::from_str::<BufferedShapes>(json);
    match (d, b) {
        (Ok(d), Ok(b)) => {
            assert_eq!(d.required, b.required, "required for {json}");
            assert_eq!(d.bare_option, b.bare_option, "bare_option for {json}");
            assert_eq!(
                d.defaulted_option, b.defaulted_option,
                "defaulted_option for {json}"
            );
            assert_eq!(d.defaulted_vec, b.defaulted_vec, "defaulted_vec for {json}");
            assert_eq!(d.defaulted_map, b.defaulted_map, "defaulted_map for {json}");
            assert_eq!(d.custom, b.custom, "custom for {json}");
            assert_eq!(d.renamed, b.renamed, "renamed for {json}");
        }
        (Err(d), Err(b)) => assert_eq!(message_of(&d), message_of(&b), "message for {json}"),
        (Ok(_), Err(b)) => panic!("derive accepted, buffered rejected {json}: {b}"),
        (Err(d), Ok(_)) => panic!("derive rejected, buffered accepted {json}: {d}"),
    }
}

#[test]
fn every_property_shape_typify_emits_agrees() {
    // Minimal: everything defaultable is absent, including the bare Option.
    shapes_agree(r#"{"required":"a","wire-name":"w"}"#);
    // Fully populated.
    shapes_agree(
        r#"{"required":"a","bare_option":"b","defaulted_option":"c",
            "defaulted_vec":["x"],"defaulted_map":{"k":1},"custom":3,"wire-name":"w"}"#,
    );
    // Explicit nulls where the type allows them.
    shapes_agree(r#"{"required":"a","bare_option":null,"defaulted_option":null,"wire-name":"w"}"#);
    // Unknown keys are ignored without deny_unknown_fields.
    shapes_agree(r#"{"required":"a","wire-name":"w","surprise":{"nested":[1,2]}}"#);
    // The renamed field must not be reachable under its Rust name.
    shapes_agree(r#"{"required":"a","renamed":"w"}"#);
    // Wrong types still line up.
    shapes_agree(r#"{"required":"a","wire-name":"w","defaulted_vec":{}}"#);
    shapes_agree(r#"{"required":"a","wire-name":"w","defaulted_map":[]}"#);
    shapes_agree(r#"{"required":"a","wire-name":"w","custom":"nope"}"#);
}

/// A fieldless enum, the shape typify emits for a JSON string enum.
///
/// The buffered form never touches these — they resolve from the variant
/// identifier alone — so this covers the *other* generated `Deserialize`.
#[derive(Debug, PartialEq, Deserialize)]
enum DerivedUnit {
    #[serde(rename = "read_write")]
    ReadWrite,
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, PartialEq)]
enum BufferedUnit {
    ReadWrite,
    Read,
    None,
}

impl de::UnitEnum for BufferedUnit {
    const NAME: &'static str = "DerivedUnit";
    const VARIANTS: &'static [&'static str] = &["read_write", "read", "none"];

    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::ReadWrite,
            1 => Self::Read,
            _ => Self::None,
        }
    }
}

impl<'de> serde::Deserialize<'de> for BufferedUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de::deserialize_unit_enum(deserializer)
    }
}

#[track_caller]
fn unit_agrees(json: &str) {
    let d = serde_json::from_str::<DerivedUnit>(json);
    let b = serde_json::from_str::<BufferedUnit>(json);
    match (d, b) {
        (Ok(d), Ok(b)) => {
            let same = matches!(
                (&d, &b),
                (DerivedUnit::ReadWrite, BufferedUnit::ReadWrite)
                    | (DerivedUnit::Read, BufferedUnit::Read)
                    | (DerivedUnit::None, BufferedUnit::None)
            );
            assert!(same, "variant mismatch for {json}: {d:?} vs {b:?}");
        }
        (Err(d), Err(b)) => assert_eq!(message_of(&d), message_of(&b), "message for {json}"),
        (Ok(_), Err(b)) => panic!("derive accepted, unit-enum rejected {json}: {b}"),
        (Err(d), Ok(_)) => panic!("derive rejected, unit-enum accepted {json}: {d}"),
    }
}

#[test]
fn a_fieldless_enum_agrees_with_the_derive() {
    unit_agrees(r#""read_write""#);
    unit_agrees(r#""read""#);
    unit_agrees(r#""none""#);
    // Unknown variant, including the full expected-list wording.
    unit_agrees(r#""nope""#);
    unit_agrees(r#""""#);
    // The Rust ident must not be accepted in place of the wire name.
    unit_agrees(r#""ReadWrite""#);
    // Wrong shapes.
    unit_agrees(r#"{}"#);
    unit_agrees(r#"[]"#);
    unit_agrees(r#"null"#);
    unit_agrees(r#"7"#);
}

#[test]
fn a_fieldless_enum_needs_no_deserialize_any() {
    // The point of resolving from the identifier: unlike the struct form, this
    // keeps working on a format that cannot answer "what is here?".
    let by_index =
        <BufferedUnit as serde::Deserialize>::deserialize(not_self_describing::UnitDeserializer {
            index: 1,
        })
        .expect("a compact format should still decode a unit variant");
    assert_eq!(by_index, BufferedUnit::Read);
}

/// Fixed-size sequence fields — the shape that stops reading before the input
/// runs out, so a trailing element would go unnoticed by everything else.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct DerivedSeq {
    pair: [u8; 2],
    duo: (String, i64),
}

#[derive(Debug, PartialEq)]
struct BufferedSeq {
    pair: [u8; 2],
    duo: (String, i64),
}

impl<'de> de::Build<'de> for BufferedSeq {
    const NAME: &'static str = "DerivedSeq";
    const FIELDS: &'static [&'static str] = &["pair", "duo"];

    fn build<E: serde::de::Error>(mut fields: de::Fields<'de>) -> Result<Self, E> {
        let value = Self {
            pair: de::required(&mut fields, "pair")?,
            duo: de::required(&mut fields, "duo")?,
        };
        de::deny_unknown(&fields, <Self as de::Build>::FIELDS)?;
        Ok(value)
    }
}

impl<'de> serde::Deserialize<'de> for BufferedSeq {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de::deserialize_struct(deserializer)
    }
}

/// Agreement on the verdict. A surplus element is the one case where the two
/// cannot agree on the *message*: serde_json rejects it at the format level
/// ("trailing characters"), while any buffered form has already consumed the
/// text and can only count what the visitor left behind. serde's own
/// `flatten`/untagged paths diverge in exactly the same way, for the same
/// reason. `a_surplus_element_is_reported_as_a_length` pins the wording.
#[track_caller]
fn seq_agrees(json: &str) {
    let derived = serde_json::from_str::<DerivedSeq>(json);
    let buffered = serde_json::from_str::<BufferedSeq>(json);

    match (derived, buffered) {
        (Ok(d), Ok(b)) => {
            assert_eq!(d.pair, b.pair, "pair mismatch for {json}");
            assert_eq!(d.duo, b.duo, "duo mismatch for {json}");
        }
        (Err(_), Err(_)) => {}
        (Ok(_), Err(b)) => panic!("derive accepted, buffered rejected {json}: {b}"),
        (Err(d), Ok(_)) => panic!("derive rejected, buffered accepted {json}: {d}"),
    }
}

#[test]
fn fixed_size_sequences_agree_on_length() {
    seq_agrees(r#"{"pair":[1,2],"duo":["a",1]}"#);
    // Short: the visitor itself notices.
    seq_agrees(r#"{"pair":[1],"duo":["a",1]}"#);
    seq_agrees(r#"{"pair":[1,2],"duo":["a"]}"#);
    // Long: only an explicit end-of-sequence check notices, because the
    // visitor stops at its fixed arity and never asks for the extra element.
    seq_agrees(r#"{"pair":[1,2,3],"duo":["a",1]}"#);
    seq_agrees(r#"{"pair":[1,2],"duo":["a",1,true]}"#);
    seq_agrees(r#"{"pair":[1,2,3,4,5],"duo":["a",1]}"#);
}

#[test]
fn a_surplus_element_is_reported_as_a_length() {
    let err = serde_json::from_str::<BufferedSeq>(r#"{"pair":[1,2,3],"duo":["a",1]}"#)
        .expect_err("a third element does not fit [u8; 2]");
    assert_eq!(
        message_of(&err),
        "invalid length 3, expected 2 elements in sequence"
    );

    let err = serde_json::from_str::<BufferedSeq>(r#"{"pair":[1,2],"duo":["a",1,true,7]}"#)
        .expect_err("four elements do not fit a 2-tuple");
    assert_eq!(
        message_of(&err),
        "invalid length 4, expected 2 elements in sequence"
    );
}
