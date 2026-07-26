//! Differential test for the emitted hand-written `Serialize`.
//!
//! Unlike the deserialize side there is no runtime to share, so this compares
//! serde's derive against a hand-written impl in exactly the shape
//! [`TypeEntry::output_struct`] emits: same container name, same wire field
//! names, same `skip_serializing_if` predicates, same field order.
//!
//! Serializing cannot fail differently between the two, so the whole contract
//! is the bytes.

use serde::Serialize;

/// Derived reference implementation, carrying every property shape
/// `generate_serde_attr` can emit on the serialize side.
#[derive(Serialize)]
#[serde(rename = "Wire")]
struct DerivedSer {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    nickname: Option<String>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    tags: Vec<String>,
    #[serde(default)]
    count: i64,
}

/// The same contract, written the way the generator emits it.
struct ManualSer {
    name: String,
    kind: String,
    nickname: Option<String>,
    tags: Vec<String>,
    count: i64,
}

impl Serialize for ManualSer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let len = 1usize
            + 1
            + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.nickname))
            + ::std::primitive::usize::from(!::std::vec::Vec::is_empty(&self.tags))
            + 1;
        let mut state = serde::Serializer::serialize_struct(serializer, "Wire", len)?;
        serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
        serde::ser::SerializeStruct::serialize_field(&mut state, "type", &self.kind)?;
        if !::std::option::Option::is_none(&self.nickname) {
            serde::ser::SerializeStruct::serialize_field(&mut state, "nickname", &self.nickname)?;
        }
        if !::std::vec::Vec::is_empty(&self.tags) {
            serde::ser::SerializeStruct::serialize_field(&mut state, "tags", &self.tags)?;
        }
        serde::ser::SerializeStruct::serialize_field(&mut state, "count", &self.count)?;
        serde::ser::SerializeStruct::end(state)
    }
}

#[track_caller]
fn agree(nickname: Option<&str>, tags: &[&str], count: i64) {
    let name = "a".to_string();
    let kind = "k".to_string();
    let nickname = nickname.map(str::to_string);
    let tags = tags.iter().map(|t| (*t).to_string()).collect::<Vec<_>>();
    let derived = DerivedSer {
        name: name.clone(),
        kind: kind.clone(),
        nickname: nickname.clone(),
        tags: tags.clone(),
        count,
    };
    let manual = ManualSer {
        name,
        kind,
        nickname,
        tags,
        count,
    };
    // Byte equality, not `Value` equality: key order is part of the contract.
    assert_eq!(
        serde_json::to_string(&derived).unwrap(),
        serde_json::to_string(&manual).unwrap(),
    );
}

#[test]
fn every_property_shape_serializes_identically() {
    // Both skippable fields present.
    agree(Some("n"), &["x", "y"], 7);
    // Each skipped in turn — the field must vanish, not become null.
    agree(None, &["x"], 7);
    agree(Some("n"), &[], 7);
    // Both skipped: exercises the smallest declared length.
    agree(None, &[], 0);
    // A present-but-empty string is not skippable, unlike an empty Vec.
    agree(Some(""), &[""], -1);
}

/// The declared struct length has to match the number of fields actually
/// emitted. `serde_json` ignores it, so a wrong count would go unnoticed there
/// while corrupting a length-prefixed format.
#[test]
fn the_declared_length_matches_the_fields_emitted() {
    use std::cell::RefCell;

    #[derive(Default)]
    struct Counts {
        declared: usize,
        emitted: usize,
    }

    thread_local! {
        static COUNTS: RefCell<Counts> = RefCell::new(Counts::default());
    }

    struct CountingSerializer;

    struct CountingStruct;

    impl serde::Serializer for CountingSerializer {
        type Ok = ();
        type Error = serde_json::Error;
        type SerializeStruct = CountingStruct;
        type SerializeSeq = serde::ser::Impossible<(), serde_json::Error>;
        type SerializeTuple = serde::ser::Impossible<(), serde_json::Error>;
        type SerializeTupleStruct = serde::ser::Impossible<(), serde_json::Error>;
        type SerializeTupleVariant = serde::ser::Impossible<(), serde_json::Error>;
        type SerializeMap = serde::ser::Impossible<(), serde_json::Error>;
        type SerializeStructVariant = serde::ser::Impossible<(), serde_json::Error>;
        fn serialize_struct(
            self,
            _name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            COUNTS.with(|c| c.borrow_mut().declared = len);
            Ok(CountingStruct)
        }
        fn serialize_bool(self, _: bool) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_i8(self, _: i8) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_i16(self, _: i16) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_i32(self, _: i32) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_i64(self, _: i64) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_u8(self, _: u8) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_u16(self, _: u16) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_u32(self, _: u32) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_u64(self, _: u64) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_f32(self, _: f32) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_f64(self, _: f64) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_char(self, _: char) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_str(self, _: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_bytes(self, _: &[u8]) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_none(self) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_unit(self) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_unit_struct(self, _: &'static str) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_unit_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_newtype_struct<T: ?Sized + Serialize>(
            self,
            _: &'static str,
            _: &T,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_newtype_variant<T: ?Sized + Serialize>(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: &T,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
        fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            unimplemented!("only structs are measured here")
        }
        fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
            unimplemented!("only structs are measured here")
        }
        fn serialize_tuple_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            unimplemented!("only structs are measured here")
        }
        fn serialize_tuple_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            unimplemented!("only structs are measured here")
        }
        fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            unimplemented!("only structs are measured here")
        }
        fn serialize_struct_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            unimplemented!("only structs are measured here")
        }
    }

    impl serde::ser::SerializeStruct for CountingStruct {
        type Ok = ();
        type Error = serde_json::Error;
        fn serialize_field<T: ?Sized + Serialize>(
            &mut self,
            _key: &'static str,
            _value: &T,
        ) -> Result<(), Self::Error> {
            COUNTS.with(|c| c.borrow_mut().emitted += 1);
            Ok(())
        }
        fn end(self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    for (nickname, tags) in [
        (Some("n".to_string()), vec!["x".to_string()]),
        (None, vec!["x".to_string()]),
        (Some("n".to_string()), Vec::new()),
        (None, Vec::new()),
    ] {
        COUNTS.with(|c| *c.borrow_mut() = Counts::default());
        let value = ManualSer {
            name: "a".to_string(),
            kind: "k".to_string(),
            nickname,
            tags,
            count: 1,
        };
        value.serialize(CountingSerializer).unwrap();
        COUNTS.with(|c| {
            let c = c.borrow();
            assert_eq!(
                c.declared, c.emitted,
                "declared struct length disagrees with the fields emitted"
            );
        });
    }
}
