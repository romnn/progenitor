#[allow(unused_imports)]
use progenitor_client::{encode_path, ClientHooks, OperationInfo, RequestBuilderExt};
#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, ClientInfo, Error, ResponseValue};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    /// Error types.
    pub mod error {
        /// Error from a `TryFrom` or `FromStr` implementation.
        pub struct ConversionError(::std::borrow::Cow<'static, str>);
        impl ::std::error::Error for ConversionError {}
        impl ::std::fmt::Display for ConversionError {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl ::std::fmt::Debug for ConversionError {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Debug::fmt(&self.0, f)
            }
        }

        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }

        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }

    /// Support for the generated `Deserialize` impls.
    ///
    /// Emitted only for
    /// [`DeserializeImpl::Buffered`](crate::DeserializeImpl::Buffered);
    /// see that variant for what the generated impls do with it.
    ///
    /// Not public API. It is `pub(crate)` so that changing the
    /// runtime is never a breaking change for the crate this was
    /// generated into.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub(crate) mod de {
        //! The `types::de` runtime, emitted verbatim into each generated crate
        //! that
        //! uses [`DeserializeImpl::Buffered`](crate::DeserializeImpl::Buffered).
        //!
        //! Generated `Deserialize` impls are two parts: a tiny generic
        //! `deserialize<D>`
        //! that drives `D::deserialize_struct` with the shared
        //! [`StructVisitor`] below,
        //! and a fully monomorphic `__build` that reads the buffered fields.
        //! The point
        //! is that only the second part is per-type, and it is not generic over
        //! the
        //! `Deserializer`, so rustc type-checks one small body per struct
        //! instead of
        //! serde_derive's nine (mostly generic) ones.
        //!
        //! `serde`'s own equivalent lives in `serde::__private::de`, but that
        //! module's
        //! name is suffixed with serde's patch version by its build script
        //! (`pub mod
        //! __private$$`), so no generator can name it — hence this copy.
        //!
        //! This file is not compiled as part of `typify-impl`; it is
        //! `include_str!`d and
        //! re-parsed into the generated output. Keeping it as ordinary Rust
        //! rather than
        //! a `quote!` block means it can be edited, formatted and tested
        //! directly.
        use serde::de::{
            self, Deserialize, DeserializeSeed, Deserializer, EnumAccess, IntoDeserializer,
            MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
        };
        use std::fmt;
        use std::marker::PhantomData;
        /// A format-agnostic buffered value.
        ///
        /// The map arm is an association list rather than a map so that
        /// duplicate keys
        /// survive buffering: `serde_derive` rejects `{"a":1,"a":2}` and
        /// collapsing
        /// into a `HashMap`/`serde_json::Map` here would silently accept it.
        #[derive(Debug, Clone)]
        pub enum Content<'de> {
            Bool(bool),
            U8(u8),
            U16(u16),
            U32(u32),
            U64(u64),
            I8(i8),
            I16(i16),
            I32(i32),
            I64(i64),
            F32(f32),
            F64(f64),
            Char(char),
            String(String),
            Str(&'de str),
            ByteBuf(Vec<u8>),
            Bytes(&'de [u8]),
            None,
            Some(Box<Content<'de>>),
            Unit,
            Newtype(Box<Content<'de>>),
            Seq(Vec<Content<'de>>),
            Map(Vec<(Content<'de>, Content<'de>)>),
        }

        impl<'de> Content<'de> {
            /// The `Unexpected` describing this value, for error messages.
            ///
            /// `Unit` maps to `Unexpected::Other("null")` rather than
            /// `Unexpected::Unit` so messages read `invalid type: null,
            /// expected …`
            /// the way `serde_json` renders them, instead of `unit value`.
            fn unexpected(&self) -> Unexpected<'_> {
                match self {
                    Content::Bool(b) => Unexpected::Bool(*b),
                    Content::U8(n) => Unexpected::Unsigned(u64::from(*n)),
                    Content::U16(n) => Unexpected::Unsigned(u64::from(*n)),
                    Content::U32(n) => Unexpected::Unsigned(u64::from(*n)),
                    Content::U64(n) => Unexpected::Unsigned(*n),
                    Content::I8(n) => Unexpected::Signed(i64::from(*n)),
                    Content::I16(n) => Unexpected::Signed(i64::from(*n)),
                    Content::I32(n) => Unexpected::Signed(i64::from(*n)),
                    Content::I64(n) => Unexpected::Signed(*n),
                    Content::F32(f) => Unexpected::Float(f64::from(*f)),
                    Content::F64(f) => Unexpected::Float(*f),
                    Content::Char(c) => Unexpected::Char(*c),
                    Content::String(s) => Unexpected::Str(s),
                    Content::Str(s) => Unexpected::Str(s),
                    Content::ByteBuf(b) => Unexpected::Bytes(b),
                    Content::Bytes(b) => Unexpected::Bytes(b),
                    Content::None | Content::Some(_) => Unexpected::Option,
                    Content::Unit => Unexpected::Other("null"),
                    Content::Newtype(_) => Unexpected::NewtypeStruct,
                    Content::Seq(_) => Unexpected::Seq,
                    Content::Map(_) => Unexpected::Map,
                }
            }
            /// The string form of a buffered map key, used to match field
            /// names.
            fn as_key_str(&self) -> Option<&str> {
                match self {
                    Content::Str(s) => Some(s),
                    Content::String(s) => Some(s),
                    Content::Bytes(b) => std::str::from_utf8(b).ok(),
                    Content::ByteBuf(b) => std::str::from_utf8(b).ok(),
                    _ => None,
                }
            }
        }

        struct ContentVisitor<'de> {
            marker: std::marker::PhantomData<Content<'de>>,
        }

        impl<'de> Visitor<'de> for ContentVisitor<'de> {
            type Value = Content<'de>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("any value")
            }
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(Content::Bool(v))
            }
            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E> {
                Ok(Content::I8(v))
            }
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E> {
                Ok(Content::I16(v))
            }
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> {
                Ok(Content::I32(v))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(Content::I64(v))
            }
            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E> {
                Ok(Content::U8(v))
            }
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E> {
                Ok(Content::U16(v))
            }
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> {
                Ok(Content::U32(v))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(Content::U64(v))
            }
            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> {
                Ok(Content::F32(v))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(Content::F64(v))
            }
            fn visit_char<E>(self, v: char) -> Result<Self::Value, E> {
                Ok(Content::Char(v))
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(Content::Unit)
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(Content::String(v.to_string()))
            }
            fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value, E> {
                Ok(Content::Str(v))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(Content::String(v))
            }
            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
                Ok(Content::ByteBuf(v.to_vec()))
            }
            fn visit_borrowed_bytes<E: de::Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
                Ok(Content::Bytes(v))
            }
            fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
                Ok(Content::ByteBuf(v))
            }
            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(Content::None)
            }
            fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
                Content::deserialize(d).map(|c| Content::Some(Box::new(c)))
            }
            fn visit_newtype_struct<D: Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                Content::deserialize(d).map(|c| Content::Newtype(Box::new(c)))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut items = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(item) = seq.next_element()? {
                    items.push(item);
                }
                Ok(Content::Seq(items))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));
                while let Some(kv) = map.next_entry()? {
                    entries.push(kv);
                }
                Ok(Content::Map(entries))
            }
            fn visit_enum<A: EnumAccess<'de>>(self, _: A) -> Result<Self::Value, A::Error> {
                Err(de::Error::custom("unexpected enum while buffering a value"))
            }
        }

        impl<'de> Deserialize<'de> for Content<'de> {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                deserializer.deserialize_any(ContentVisitor {
                    marker: std::marker::PhantomData,
                })
            }
        }

        /// Deserializer that replays a buffered [`Content`].
        pub struct ContentDeserializer<'de, E> {
            content: Content<'de>,
            marker: std::marker::PhantomData<E>,
        }

        impl<'de, E: de::Error> ContentDeserializer<'de, E> {
            pub fn new(content: Content<'de>) -> Self {
                Self {
                    content,
                    marker: std::marker::PhantomData,
                }
            }
            fn invalid_type<T>(self, exp: &dyn de::Expected) -> Result<T, E> {
                Err(E::invalid_type(self.content.unexpected(), exp))
            }
        }

        impl<'de, E: de::Error> IntoDeserializer<'de, E> for ContentDeserializer<'de, E> {
            type Deserializer = Self;
            fn into_deserializer(self) -> Self {
                self
            }
        }

        macro_rules ! forward_number { ($ ($ method : ident => $ visit : ident ,) *) => { $ (fn $ method < V : Visitor <'de >> (self , visitor : V) -> Result < V :: Value , E > { self . deserialize_any (visitor) }) * } ; }
        impl<'de, E: de::Error> Deserializer<'de> for ContentDeserializer<'de, E> {
            type Error = E;
            fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, E> {
                match self.content {
                    Content::Bool(v) => visitor.visit_bool(v),
                    Content::U8(v) => visitor.visit_u8(v),
                    Content::U16(v) => visitor.visit_u16(v),
                    Content::U32(v) => visitor.visit_u32(v),
                    Content::U64(v) => visitor.visit_u64(v),
                    Content::I8(v) => visitor.visit_i8(v),
                    Content::I16(v) => visitor.visit_i16(v),
                    Content::I32(v) => visitor.visit_i32(v),
                    Content::I64(v) => visitor.visit_i64(v),
                    Content::F32(v) => visitor.visit_f32(v),
                    Content::F64(v) => visitor.visit_f64(v),
                    Content::Char(v) => visitor.visit_char(v),
                    Content::String(v) => visitor.visit_string(v),
                    Content::Str(v) => visitor.visit_borrowed_str(v),
                    Content::ByteBuf(v) => visitor.visit_byte_buf(v),
                    Content::Bytes(v) => visitor.visit_borrowed_bytes(v),
                    Content::Unit => visitor.visit_unit(),
                    Content::None => visitor.visit_none(),
                    Content::Some(v) => visitor.visit_some(ContentDeserializer::new(*v)),
                    Content::Newtype(v) => {
                        visitor.visit_newtype_struct(ContentDeserializer::new(*v))
                    }
                    Content::Seq(v) => {
                        let mut seq = SeqReplay::new(v);
                        let value = visitor.visit_seq(&mut seq)?;
                        seq.end()?;
                        Ok(value)
                    }
                    Content::Map(v) => {
                        let mut map = MapReplay::new(v);
                        let value = visitor.visit_map(&mut map)?;
                        map.end()?;
                        Ok(value)
                    }
                }
            }
            fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, E> {
                match self.content {
                    Content::None | Content::Unit => visitor.visit_none(),
                    Content::Some(v) => visitor.visit_some(ContentDeserializer::new(*v)),
                    other => visitor.visit_some(ContentDeserializer::new(other)),
                }
            }
            fn deserialize_newtype_struct<V: Visitor<'de>>(
                self,
                _name: &'static str,
                visitor: V,
            ) -> Result<V::Value, E> {
                match self.content {
                    Content::Newtype(v) => {
                        visitor.visit_newtype_struct(ContentDeserializer::new(*v))
                    }
                    other => visitor.visit_newtype_struct(ContentDeserializer::new(other)),
                }
            }
            fn deserialize_enum<V: Visitor<'de>>(
                self,
                _name: &'static str,
                _variants: &'static [&'static str],
                visitor: V,
            ) -> Result<V::Value, E> {
                let (variant, value) = match self.content {
                    Content::Map(entries) => {
                        let mut iter = entries.into_iter();
                        let (variant, value) = match iter.next() {
                            Some(kv) => kv,
                            None => {
                                return Err(E::invalid_value(
                                    Unexpected::Map,
                                    &"map with a single key",
                                ));
                            }
                        };
                        if iter.next().is_some() {
                            return Err(E::invalid_value(
                                Unexpected::Map,
                                &"map with a single key",
                            ));
                        }
                        (variant, Some(value))
                    }
                    s @ (Content::String(_) | Content::Str(_)) => (s, None),
                    other => return Err(E::invalid_type(other.unexpected(), &"string or map")),
                };
                visitor.visit_enum(EnumReplay {
                    variant,
                    value,
                    marker: std::marker::PhantomData,
                })
            }
            fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, E> {
                match self.content {
                    Content::Bool(v) => visitor.visit_bool(v),
                    _ => self.invalid_type(&visitor),
                }
            }
            forward_number! { deserialize_i8 => visit_i8 , deserialize_i16 => visit_i16 , deserialize_i32 => visit_i32 , deserialize_i64 => visit_i64 , deserialize_u8 => visit_u8 , deserialize_u16 => visit_u16 , deserialize_u32 => visit_u32 , deserialize_u64 => visit_u64 , deserialize_f32 => visit_f32 , deserialize_f64 => visit_f64 , deserialize_char => visit_char , deserialize_str => visit_str , deserialize_string => visit_string , deserialize_bytes => visit_bytes , deserialize_byte_buf => visit_byte_buf , deserialize_unit => visit_unit , deserialize_seq => visit_seq , deserialize_map => visit_map , deserialize_identifier => visit_identifier , deserialize_ignored_any => visit_ignored_any , }
            fn deserialize_unit_struct<V: Visitor<'de>>(
                self,
                _name: &'static str,
                visitor: V,
            ) -> Result<V::Value, E> {
                self.deserialize_any(visitor)
            }
            fn deserialize_tuple<V: Visitor<'de>>(
                self,
                _len: usize,
                visitor: V,
            ) -> Result<V::Value, E> {
                self.deserialize_any(visitor)
            }
            fn deserialize_tuple_struct<V: Visitor<'de>>(
                self,
                _name: &'static str,
                _len: usize,
                visitor: V,
            ) -> Result<V::Value, E> {
                self.deserialize_any(visitor)
            }
            fn deserialize_struct<V: Visitor<'de>>(
                self,
                _name: &'static str,
                _fields: &'static [&'static str],
                visitor: V,
            ) -> Result<V::Value, E> {
                self.deserialize_any(visitor)
            }
        }

        struct SeqReplay<'de, E> {
            iter: std::vec::IntoIter<Content<'de>>,
            taken: usize,
            marker: std::marker::PhantomData<E>,
        }

        impl<'de, E: de::Error> SeqReplay<'de, E> {
            fn new(content: Vec<Content<'de>>) -> Self {
                Self {
                    iter: content.into_iter(),
                    taken: 0,
                    marker: std::marker::PhantomData,
                }
            }
            /// Reject whatever the visitor did not consume, in serde's own
            /// wording for
            /// a replayed sequence.
            fn end(self) -> Result<(), E> {
                let remaining = self.iter.count();
                if remaining == 0 {
                    Ok(())
                } else {
                    Err(E::invalid_length(
                        self.taken + remaining,
                        &ExpectedIn::seq(self.taken),
                    ))
                }
            }
        }

        impl<'de, E: de::Error> SeqAccess<'de> for &mut SeqReplay<'de, E> {
            type Error = E;
            fn next_element_seed<T: DeserializeSeed<'de>>(
                &mut self,
                seed: T,
            ) -> Result<Option<T::Value>, E> {
                match self.iter.next() {
                    Some(c) => {
                        self.taken += 1;
                        seed.deserialize(ContentDeserializer::new(c)).map(Some)
                    }
                    None => Ok(None),
                }
            }
            fn size_hint(&self) -> Option<usize> {
                Some(self.iter.len())
            }
        }

        struct MapReplay<'de, E> {
            iter: std::vec::IntoIter<(Content<'de>, Content<'de>)>,
            value: Option<Content<'de>>,
            taken: usize,
            marker: std::marker::PhantomData<E>,
        }

        impl<'de, E: de::Error> MapReplay<'de, E> {
            fn new(content: Vec<(Content<'de>, Content<'de>)>) -> Self {
                Self {
                    iter: content.into_iter(),
                    value: None,
                    taken: 0,
                    marker: std::marker::PhantomData,
                }
            }
            /// The map counterpart of [`SeqReplay::end`]. Struct visitors drain
            /// the
            /// map, so this only bites for a visitor with a fixed shape — but
            /// the
            /// asymmetry is not worth relying on.
            fn end(self) -> Result<(), E> {
                let remaining = self.iter.count();
                if remaining == 0 {
                    Ok(())
                } else {
                    Err(E::invalid_length(
                        self.taken + remaining,
                        &ExpectedIn::map(self.taken),
                    ))
                }
            }
        }

        impl<'de, E: de::Error> MapAccess<'de> for &mut MapReplay<'de, E> {
            type Error = E;
            fn next_key_seed<K: DeserializeSeed<'de>>(
                &mut self,
                seed: K,
            ) -> Result<Option<K::Value>, E> {
                match self.iter.next() {
                    Some((k, v)) => {
                        self.taken += 1;
                        self.value = Some(v);
                        seed.deserialize(ContentDeserializer::new(k)).map(Some)
                    }
                    None => Ok(None),
                }
            }
            fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, E> {
                let value = self
                    .value
                    .take()
                    .expect("next_value_seed before next_key_seed");
                seed.deserialize(ContentDeserializer::new(value))
            }
            fn size_hint(&self) -> Option<usize> {
                Some(self.iter.len())
            }
        }

        /// serde's phrasing for "the visitor stopped before the input did".
        struct ExpectedIn {
            taken: usize,
            container: &'static str,
        }

        impl ExpectedIn {
            fn seq(taken: usize) -> Self {
                Self {
                    taken,
                    container: "sequence",
                }
            }
            fn map(taken: usize) -> Self {
                Self {
                    taken,
                    container: "map",
                }
            }
        }

        impl de::Expected for ExpectedIn {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.taken {
                    1 => write!(f, "1 element in {}", self.container),
                    n => write!(f, "{n} elements in {}", self.container),
                }
            }
        }

        struct EnumReplay<'de, E> {
            variant: Content<'de>,
            value: Option<Content<'de>>,
            marker: std::marker::PhantomData<E>,
        }

        impl<'de, E: de::Error> EnumAccess<'de> for EnumReplay<'de, E> {
            type Error = E;
            type Variant = VariantReplay<'de, E>;
            fn variant_seed<V: DeserializeSeed<'de>>(
                self,
                seed: V,
            ) -> Result<(V::Value, Self::Variant), E> {
                let variant = seed.deserialize(ContentDeserializer::new(self.variant))?;
                Ok((
                    variant,
                    VariantReplay {
                        value: self.value,
                        marker: std::marker::PhantomData,
                    },
                ))
            }
        }

        struct VariantReplay<'de, E> {
            value: Option<Content<'de>>,
            marker: std::marker::PhantomData<E>,
        }

        impl<'de, E: de::Error> VariantAccess<'de> for VariantReplay<'de, E> {
            type Error = E;
            fn unit_variant(self) -> Result<(), E> {
                match self.value {
                    None | Some(Content::Unit) => Ok(()),
                    Some(other) => Err(E::invalid_type(other.unexpected(), &"unit variant")),
                }
            }
            fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, E> {
                match self.value {
                    Some(v) => seed.deserialize(ContentDeserializer::new(v)),
                    None => Err(E::invalid_type(Unexpected::UnitVariant, &"newtype variant")),
                }
            }
            fn tuple_variant<V: Visitor<'de>>(
                self,
                _len: usize,
                visitor: V,
            ) -> Result<V::Value, E> {
                match self.value {
                    Some(v) => ContentDeserializer::new(v).deserialize_any(visitor),
                    None => Err(E::invalid_type(Unexpected::UnitVariant, &"tuple variant")),
                }
            }
            fn struct_variant<V: Visitor<'de>>(
                self,
                _fields: &'static [&'static str],
                visitor: V,
            ) -> Result<V::Value, E> {
                match self.value {
                    Some(v) => ContentDeserializer::new(v).deserialize_any(visitor),
                    None => Err(E::invalid_type(Unexpected::UnitVariant, &"struct variant")),
                }
            }
        }

        /// The expectation `serde_derive` names when a sequence is too short:
        /// `struct Foo with 3 elements`, distinct from the visitor's own
        /// `expecting`.
        struct StructLen {
            name: &'static str,
            len: usize,
        }

        impl de::Expected for StructLen {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "struct {} with {} elements", self.name, self.len)
            }
        }

        /// The buffered fields of one struct, in wire order.
        pub type Fields<'de> = Vec<(Content<'de>, Content<'de>)>;
        /// Visitor a generated `deserialize` hands to
        /// `Deserializer::deserialize_struct`.
        ///
        /// Implements both `visit_map` and `visit_seq` so non-self-describing
        /// formats
        /// (bincode, postcard, MessagePack in compact mode) keep working: a
        /// sequence is
        /// zipped against the struct's static field list here, in shared code,
        /// so
        /// `__build` only ever sees a map and pays nothing per struct for it.
        /// Per-type half of a buffered `Deserialize`, implemented by generated
        /// types.
        ///
        /// Splitting it out this way is what keeps the emitted code to two
        /// bodies per
        /// struct: everything that walks the input lives in the generic
        /// [`StructVisitor`] below and is type-checked once, no matter how many
        /// types
        /// implement this.
        pub trait Build<'de>: Sized {
            /// The name the format sees, i.e. after any container rename.
            const NAME: &'static str;
            /// The wire names of the fields, in declaration order.
            const FIELDS: &'static [&'static str];
            /// Assemble the value from already-buffered fields.
            ///
            /// Deliberately generic only over the error type, never over the
            /// `Deserializer` — this is the half that repeats per type.
            fn build<E: de::Error>(fields: Fields<'de>) -> Result<Self, E>;
        }

        /// Drives `T`'s buffered deserialization. The whole per-type
        /// `deserialize`.
        pub fn deserialize_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
        where
            T: Build<'de>,
            D: Deserializer<'de>,
        {
            deserializer.deserialize_struct(T::NAME, T::FIELDS, StructVisitor::<T>(PhantomData))
        }

        /// Visitor handed to `deserialize_struct`.
        ///
        /// Implements both `visit_map` and `visit_seq` so non-self-describing
        /// formats
        /// keep working: a sequence is zipped against the struct's static field
        /// list
        /// here, in shared code, so `build` only ever sees a map.
        ///
        /// `build` is called *inside* the visitor rather than after it returns,
        /// which
        /// is what keeps a format's positional error fixup (serde_json's
        /// line/column)
        /// applying to whatever `build` reports.
        pub struct StructVisitor<T>(pub PhantomData<T>);
        impl<'de, T: Build<'de>> Visitor<'de> for StructVisitor<T> {
            type Value = T;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "struct {}", T::NAME)
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(T::FIELDS.len()));
                while let Some(kv) = map.next_entry()? {
                    entries.push(kv);
                }
                T::build(entries)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::with_capacity(T::FIELDS.len());
                for (index, name) in T::FIELDS.iter().enumerate() {
                    match seq.next_element::<Content<'de>>()? {
                        Some(value) => entries.push((Content::Str(name), value)),
                        None => {
                            return Err(de::Error::invalid_length(
                                index,
                                &StructLen {
                                    name: T::NAME,
                                    len: T::FIELDS.len(),
                                },
                            ));
                        }
                    }
                }
                T::build(entries)
            }
        }

        /// Take the value for `name`, erroring on a duplicate the way the
        /// derive does.
        ///
        /// Returns the raw `Content` so the caller decides between "absent" and
        /// "present but null", which `serde_derive` distinguishes and a
        /// `HashMap`-based
        /// buffer cannot.
        pub fn take<'de, E: de::Error>(
            fields: &mut Fields<'de>,
            name: &'static str,
        ) -> Result<Option<Content<'de>>, E> {
            let mut found = None;
            let mut index = 0;
            while index < fields.len() {
                if fields[index].0.as_key_str() == Some(name) {
                    if found.is_some() {
                        return Err(E::duplicate_field(name));
                    }
                    found = Some(fields.remove(index).1);
                } else {
                    index += 1;
                }
            }
            Ok(found)
        }

        /// Per-type half of a fieldless enum's `Deserialize`.
        ///
        /// Unlike [`Build`], this involves **no buffering**: a unit variant is
        /// decided
        /// from the identifier alone, so the generated impl drives
        /// `deserialize_enum`
        /// directly and keeps working with non-self-describing formats, and any
        /// error
        /// it reports carries the format's own position.
        pub trait UnitEnum: Sized {
            /// The name the format sees, i.e. after any container rename.
            const NAME: &'static str;
            /// The wire names of the variants, in declaration order.
            const VARIANTS: &'static [&'static str];
            /// Build the variant at `index` within [`Self::VARIANTS`].
            ///
            /// Only ever called with an index this module has already
            /// bounds-checked
            /// against `VARIANTS`.
            fn from_index(index: usize) -> Self;
        }

        /// Resolves a variant identifier to its index in `T::VARIANTS`.
        struct VariantIndex<T>(usize, PhantomData<T>);
        impl<'de, T: UnitEnum> Deserialize<'de> for VariantIndex<T> {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                deserializer.deserialize_identifier(VariantIndexVisitor::<T>(PhantomData))
            }
        }

        struct VariantIndexVisitor<T>(PhantomData<T>);
        impl<'de, T: UnitEnum> Visitor<'de> for VariantIndexVisitor<T> {
            type Value = VariantIndex<T>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                match usize::try_from(value)
                    .ok()
                    .filter(|index| *index < T::VARIANTS.len())
                {
                    Some(index) => Ok(VariantIndex(index, PhantomData)),
                    None => Err(E::invalid_value(
                        Unexpected::Unsigned(value),
                        &VariantIndexRange(T::VARIANTS.len()),
                    )),
                }
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match T::VARIANTS.iter().position(|name| *name == value) {
                    Some(index) => Ok(VariantIndex(index, PhantomData)),
                    None => Err(E::unknown_variant(value, T::VARIANTS)),
                }
            }
            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                match std::str::from_utf8(value) {
                    Ok(text) => self.visit_str(text),
                    Err(_) => Err(E::invalid_value(Unexpected::Bytes(value), &self)),
                }
            }
        }

        /// `serde_derive`'s wording for an out-of-range variant index.
        struct VariantIndexRange(usize);
        impl de::Expected for VariantIndexRange {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "variant index 0 <= i < {}", self.0)
            }
        }

        struct UnitEnumVisitor<T>(PhantomData<T>);
        impl<'de, T: UnitEnum> Visitor<'de> for UnitEnumVisitor<T> {
            type Value = T;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "enum {}", T::NAME)
            }
            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
                let (index, variant) = data.variant::<VariantIndex<T>>()?;
                de::VariantAccess::unit_variant(variant)?;
                Ok(T::from_index(index.0))
            }
        }

        /// Drives `T`'s deserialization. The whole per-type `deserialize`.
        pub fn deserialize_unit_enum<'de, T, D>(deserializer: D) -> Result<T, D::Error>
        where
            T: UnitEnum,
            D: Deserializer<'de>,
        {
            deserializer.deserialize_enum(T::NAME, T::VARIANTS, UnitEnumVisitor::<T>(PhantomData))
        }

        /// The deserializer `serde_derive` hands a field that never showed up.
        ///
        /// An absent key is a `missing field` error for most types — but *not*
        /// for
        /// `Option<T>`, which sees `None`. serde gets this by asking `T` to
        /// deserialize
        /// from a source that answers `visit_none` to `deserialize_option` and
        /// errors
        /// for everything else, so the special case lives in `Option`'s own
        /// impl rather
        /// than in the generated code. A by-name lookup has to do the same, or
        /// it
        /// rejects documents the derive accepts whenever a schema marks a
        /// nullable
        /// property required.
        struct MissingField<E> {
            name: &'static str,
            marker: PhantomData<E>,
        }

        impl<'de, E: de::Error> Deserializer<'de> for MissingField<E> {
            type Error = E;
            fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, E> {
                Err(E::missing_field(self.name))
            }
            fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, E> {
                visitor.visit_none()
            }
            serde::forward_to_deserialize_any! { bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier ignored_any }
        }

        /// Deserialize a field the schema marks required.
        pub fn required<'de, T: Deserialize<'de>, E: de::Error>(
            fields: &mut Fields<'de>,
            name: &'static str,
        ) -> Result<T, E> {
            match take(fields, name)? {
                Some(content) => T::deserialize(ContentDeserializer::new(content)),
                None => T::deserialize(MissingField {
                    name,
                    marker: PhantomData,
                }),
            }
        }

        /// Deserialize a field that falls back to `Default` when the key is
        /// absent.
        ///
        /// Mirrors `#[serde(default)]`: an absent key uses the default, while a
        /// present
        /// `null` is still handed to `T`'s own impl (so `Option<T>` sees `None`
        /// and a
        /// non-optional `T` reports the same type error the derive would).
        pub fn defaulted<'de, T: Deserialize<'de> + Default, E: de::Error>(
            fields: &mut Fields<'de>,
            name: &'static str,
        ) -> Result<T, E> {
            match take(fields, name)? {
                Some(content) => T::deserialize(ContentDeserializer::new(content)),
                None => Ok(T::default()),
            }
        }

        /// Deserialize a field whose absent-value comes from a named function,
        /// as
        /// emitted for `#[serde(default = "path")]`.
        pub fn defaulted_with<'de, T: Deserialize<'de>, E: de::Error>(
            fields: &mut Fields<'de>,
            name: &'static str,
            default: fn() -> T,
        ) -> Result<T, E> {
            match take(fields, name)? {
                Some(content) => T::deserialize(ContentDeserializer::new(content)),
                None => Ok(default()),
            }
        }

        /// Reject leftover keys, as `#[serde(deny_unknown_fields)]` does.
        pub fn deny_unknown<'de, E: de::Error>(
            fields: &Fields<'de>,
            known: &'static [&'static str],
        ) -> Result<(), E> {
            match fields.first() {
                Some((key, _)) => match key.as_key_str() {
                    Some(name) => Err(E::unknown_field(name, known)),
                    None => Err(E::custom("unknown field")),
                },
                None => Ok(()),
            }
        }
    }

    ///`GetThingOrThingsId`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "oneOf": [
    ///    {
    ///      "type": "string"
    ///    },
    ///    {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  ]
    /// }
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, schemars :: JsonSchema,
    )]
    #[serde(untagged)]
    pub enum GetThingOrThingsId {
        String(::std::string::String),
        Array(::std::vec::Vec<::std::string::String>),
    }

    impl ::std::convert::From<::std::vec::Vec<::std::string::String>> for GetThingOrThingsId {
        fn from(value: ::std::vec::Vec<::std::string::String>) -> Self {
            Self::Array(value)
        }
    }

    ///`HeaderArgAcceptLanguage`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "default": "en",
    ///  "type": "string",
    ///  "enum": [
    ///    "de",
    ///    "en"
    ///  ]
    /// }
    /// ```
    /// </details>
    #[derive(
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
        schemars :: JsonSchema,
    )]
    pub enum HeaderArgAcceptLanguage {
        #[serde(rename = "de")]
        De,
        #[serde(rename = "en")]
        En,
    }

    impl self::de::UnitEnum for HeaderArgAcceptLanguage {
        const NAME: &'static str = "HeaderArgAcceptLanguage";
        const VARIANTS: &'static [&'static str] = &["de", "en"];
        fn from_index(index: usize) -> Self {
            match index {
                0usize => Self::De,
                1usize => Self::En,
                _ => unreachable!("variant index out of range"),
            }
        }
    }

    impl<'de> ::serde::Deserialize<'de> for HeaderArgAcceptLanguage {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_unit_enum(deserializer)
        }
    }

    impl ::std::fmt::Display for HeaderArgAcceptLanguage {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::De => f.write_str("de"),
                Self::En => f.write_str("en"),
            }
        }
    }

    impl ::std::str::FromStr for HeaderArgAcceptLanguage {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "de" => Ok(Self::De),
                "en" => Ok(Self::En),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for HeaderArgAcceptLanguage {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for HeaderArgAcceptLanguage {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for HeaderArgAcceptLanguage {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::default::Default for HeaderArgAcceptLanguage {
        fn default() -> Self {
            HeaderArgAcceptLanguage::En
        }
    }

    ///`ObjWithOptionArray`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "stranger-things",
    ///    "things"
    ///  ],
    ///  "properties": {
    ///    "stranger-things": {
    ///      "type": "array",
    ///      "items": {
    ///        "oneOf": [
    ///          {
    ///            "type": "null"
    ///          },
    ///          {
    ///            "allOf": [
    ///              {
    ///                "$ref": "#/components/schemas/Task"
    ///              }
    ///            ],
    ///            "oneOf": [
    ///              {}
    ///            ]
    ///          }
    ///        ]
    ///      }
    ///    },
    ///    "things": {
    ///      "type": "array",
    ///      "items": {
    ///        "oneOf": [
    ///          {
    ///            "type": "null"
    ///          },
    ///          {
    ///            "allOf": [
    ///              {
    ///                "$ref": "#/components/schemas/Task"
    ///              }
    ///            ]
    ///          }
    ///        ]
    ///      }
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct ObjWithOptionArray {
        #[serde(rename = "stranger-things")]
        pub stranger_things: ::std::vec::Vec<::std::option::Option<Task>>,
        pub things: ::std::vec::Vec<::std::option::Option<Task>>,
    }

    impl<'de> self::de::Build<'de> for ObjWithOptionArray {
        const NAME: &'static str = "ObjWithOptionArray";
        const FIELDS: &'static [&'static str] = &["stranger-things", "things"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                stranger_things: self::de::required(&mut fields, "stranger-things")?,
                things: self::de::required(&mut fields, "things")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for ObjWithOptionArray {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ObjWithOptionArray {
        pub fn builder() -> builder::ObjWithOptionArray {
            ::std::default::Default::default()
        }
    }

    ///`Task`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "name",
    ///    "output_rules",
    ///    "script",
    ///    "state"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "output_rules": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "script": {
    ///      "type": "string"
    ///    },
    ///    "state": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct Task {
        pub id: ::std::string::String,
        pub name: ::std::string::String,
        pub output_rules: ::std::vec::Vec<::std::string::String>,
        pub script: ::std::string::String,
        pub state: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for Task {
        const NAME: &'static str = "Task";
        const FIELDS: &'static [&'static str] = &["id", "name", "output_rules", "script", "state"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                id: self::de::required(&mut fields, "id")?,
                name: self::de::required(&mut fields, "name")?,
                output_rules: self::de::required(&mut fields, "output_rules")?,
                script: self::de::required(&mut fields, "script")?,
                state: self::de::required(&mut fields, "state")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for Task {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl Task {
        pub fn builder() -> builder::Task {
            ::std::default::Default::default()
        }
    }

    ///`TaskEvent`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "payload",
    ///    "seq",
    ///    "stream",
    ///    "time"
    ///  ],
    ///  "properties": {
    ///    "payload": {
    ///      "type": "string"
    ///    },
    ///    "seq": {
    ///      "type": "integer",
    ///      "format": "uint",
    ///      "minimum": 0.0
    ///    },
    ///    "stream": {
    ///      "type": "string"
    ///    },
    ///    "time": {
    ///      "type": "string",
    ///      "format": "date-time"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct TaskEvent {
        pub payload: ::std::string::String,
        pub seq: u32,
        pub stream: ::std::string::String,
        pub time: ::chrono::DateTime<::chrono::offset::Utc>,
    }

    impl<'de> self::de::Build<'de> for TaskEvent {
        const NAME: &'static str = "TaskEvent";
        const FIELDS: &'static [&'static str] = &["payload", "seq", "stream", "time"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                payload: self::de::required(&mut fields, "payload")?,
                seq: self::de::required(&mut fields, "seq")?,
                stream: self::de::required(&mut fields, "stream")?,
                time: self::de::required(&mut fields, "time")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for TaskEvent {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl TaskEvent {
        pub fn builder() -> builder::TaskEvent {
            ::std::default::Default::default()
        }
    }

    ///`TaskOutput`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "path",
    ///    "size"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "path": {
    ///      "type": "string"
    ///    },
    ///    "size": {
    ///      "type": "integer",
    ///      "format": "uint64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct TaskOutput {
        pub id: ::std::string::String,
        pub path: ::std::string::String,
        pub size: u64,
    }

    impl<'de> self::de::Build<'de> for TaskOutput {
        const NAME: &'static str = "TaskOutput";
        const FIELDS: &'static [&'static str] = &["id", "path", "size"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                id: self::de::required(&mut fields, "id")?,
                path: self::de::required(&mut fields, "path")?,
                size: self::de::required(&mut fields, "size")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for TaskOutput {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl TaskOutput {
        pub fn builder() -> builder::TaskOutput {
            ::std::default::Default::default()
        }
    }

    ///`TaskSubmit`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "default",
    ///    "name",
    ///    "script"
    ///  ],
    ///  "properties": {
    ///    "default": {
    ///      "type": "boolean"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "output_rules": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "script": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct TaskSubmit {
        pub default: bool,
        pub name: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        pub output_rules: ::std::vec::Vec<::std::string::String>,
        pub script: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for TaskSubmit {
        const NAME: &'static str = "TaskSubmit";
        const FIELDS: &'static [&'static str] = &["default", "name", "output_rules", "script"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                default: self::de::required(&mut fields, "default")?,
                name: self::de::required(&mut fields, "name")?,
                output_rules: self::de::defaulted(&mut fields, "output_rules")?,
                script: self::de::required(&mut fields, "script")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for TaskSubmit {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl TaskSubmit {
        pub fn builder() -> builder::TaskSubmit {
            ::std::default::Default::default()
        }
    }

    ///`TaskSubmitResult`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct TaskSubmitResult {
        pub id: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for TaskSubmitResult {
        const NAME: &'static str = "TaskSubmitResult";
        const FIELDS: &'static [&'static str] = &["id"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                id: self::de::required(&mut fields, "id")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for TaskSubmitResult {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl TaskSubmitResult {
        pub fn builder() -> builder::TaskSubmitResult {
            ::std::default::Default::default()
        }
    }

    ///`UploadedChunk`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    pub use self::TaskSubmitResult as UploadedChunk;
    ///`UserCreate`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "name": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct UserCreate {
        pub name: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for UserCreate {
        const NAME: &'static str = "UserCreate";
        const FIELDS: &'static [&'static str] = &["name"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                name: self::de::required(&mut fields, "name")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for UserCreate {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl UserCreate {
        pub fn builder() -> builder::UserCreate {
            ::std::default::Default::default()
        }
    }

    ///`UserCreateResult`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "name",
    ///    "token"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "token": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct UserCreateResult {
        pub id: ::std::string::String,
        pub name: ::std::string::String,
        pub token: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for UserCreateResult {
        const NAME: &'static str = "UserCreateResult";
        const FIELDS: &'static [&'static str] = &["id", "name", "token"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                id: self::de::required(&mut fields, "id")?,
                name: self::de::required(&mut fields, "name")?,
                token: self::de::required(&mut fields, "token")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for UserCreateResult {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl UserCreateResult {
        pub fn builder() -> builder::UserCreateResult {
            ::std::default::Default::default()
        }
    }

    ///`WhoamiResult`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct WhoamiResult {
        pub id: ::std::string::String,
        pub name: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for WhoamiResult {
        const NAME: &'static str = "WhoamiResult";
        const FIELDS: &'static [&'static str] = &["id", "name"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                id: self::de::required(&mut fields, "id")?,
                name: self::de::required(&mut fields, "name")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for WhoamiResult {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl WhoamiResult {
        pub fn builder() -> builder::WhoamiResult {
            ::std::default::Default::default()
        }
    }

    ///`Worker`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "deleted",
    ///    "id",
    ///    "recycle",
    ///    "tasks"
    ///  ],
    ///  "properties": {
    ///    "deleted": {
    ///      "type": "boolean"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "instance_id": {
    ///      "type": "string"
    ///    },
    ///    "lastping": {
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "recycle": {
    ///      "type": "boolean"
    ///    },
    ///    "tasks": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WorkerTask"
    ///      }
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct Worker {
        pub deleted: bool,
        pub id: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub instance_id: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub lastping: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
        pub recycle: bool,
        pub tasks: ::std::vec::Vec<WorkerTask>,
    }

    impl<'de> self::de::Build<'de> for Worker {
        const NAME: &'static str = "Worker";
        const FIELDS: &'static [&'static str] = &[
            "deleted",
            "id",
            "instance_id",
            "lastping",
            "recycle",
            "tasks",
        ];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                deleted: self::de::required(&mut fields, "deleted")?,
                id: self::de::required(&mut fields, "id")?,
                instance_id: self::de::defaulted(&mut fields, "instance_id")?,
                lastping: self::de::defaulted(&mut fields, "lastping")?,
                recycle: self::de::required(&mut fields, "recycle")?,
                tasks: self::de::required(&mut fields, "tasks")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for Worker {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl Worker {
        pub fn builder() -> builder::Worker {
            ::std::default::Default::default()
        }
    }

    ///`WorkerAddOutput`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "chunks",
    ///    "path",
    ///    "size"
    ///  ],
    ///  "properties": {
    ///    "chunks": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "path": {
    ///      "type": "string"
    ///    },
    ///    "size": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct WorkerAddOutput {
        pub chunks: ::std::vec::Vec<::std::string::String>,
        pub path: ::std::string::String,
        pub size: i64,
    }

    impl<'de> self::de::Build<'de> for WorkerAddOutput {
        const NAME: &'static str = "WorkerAddOutput";
        const FIELDS: &'static [&'static str] = &["chunks", "path", "size"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                chunks: self::de::required(&mut fields, "chunks")?,
                path: self::de::required(&mut fields, "path")?,
                size: self::de::required(&mut fields, "size")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for WorkerAddOutput {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl WorkerAddOutput {
        pub fn builder() -> builder::WorkerAddOutput {
            ::std::default::Default::default()
        }
    }

    ///`WorkerAppendTask`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "payload",
    ///    "stream",
    ///    "time"
    ///  ],
    ///  "properties": {
    ///    "payload": {
    ///      "type": "string"
    ///    },
    ///    "stream": {
    ///      "type": "string"
    ///    },
    ///    "time": {
    ///      "type": "string",
    ///      "format": "date-time"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct WorkerAppendTask {
        pub payload: ::std::string::String,
        pub stream: ::std::string::String,
        pub time: ::chrono::DateTime<::chrono::offset::Utc>,
    }

    impl<'de> self::de::Build<'de> for WorkerAppendTask {
        const NAME: &'static str = "WorkerAppendTask";
        const FIELDS: &'static [&'static str] = &["payload", "stream", "time"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                payload: self::de::required(&mut fields, "payload")?,
                stream: self::de::required(&mut fields, "stream")?,
                time: self::de::required(&mut fields, "time")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for WorkerAppendTask {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl WorkerAppendTask {
        pub fn builder() -> builder::WorkerAppendTask {
            ::std::default::Default::default()
        }
    }

    ///`WorkerBootstrap`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "bootstrap",
    ///    "token"
    ///  ],
    ///  "properties": {
    ///    "bootstrap": {
    ///      "type": "string"
    ///    },
    ///    "token": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct WorkerBootstrap {
        pub bootstrap: ::std::string::String,
        pub token: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for WorkerBootstrap {
        const NAME: &'static str = "WorkerBootstrap";
        const FIELDS: &'static [&'static str] = &["bootstrap", "token"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                bootstrap: self::de::required(&mut fields, "bootstrap")?,
                token: self::de::required(&mut fields, "token")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for WorkerBootstrap {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl WorkerBootstrap {
        pub fn builder() -> builder::WorkerBootstrap {
            ::std::default::Default::default()
        }
    }

    ///`WorkerBootstrapResult`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    pub use self::TaskSubmitResult as WorkerBootstrapResult;
    ///`WorkerCompleteTask`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "failed"
    ///  ],
    ///  "properties": {
    ///    "failed": {
    ///      "type": "boolean"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct WorkerCompleteTask {
        pub failed: bool,
    }

    impl<'de> self::de::Build<'de> for WorkerCompleteTask {
        const NAME: &'static str = "WorkerCompleteTask";
        const FIELDS: &'static [&'static str] = &["failed"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                failed: self::de::required(&mut fields, "failed")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for WorkerCompleteTask {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl WorkerCompleteTask {
        pub fn builder() -> builder::WorkerCompleteTask {
            ::std::default::Default::default()
        }
    }

    ///`WorkerPingResult`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "poweroff"
    ///  ],
    ///  "properties": {
    ///    "poweroff": {
    ///      "type": "boolean"
    ///    },
    ///    "task": {
    ///      "$ref": "#/components/schemas/WorkerPingTask"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct WorkerPingResult {
        pub poweroff: bool,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub task: ::std::option::Option<WorkerPingTask>,
    }

    impl<'de> self::de::Build<'de> for WorkerPingResult {
        const NAME: &'static str = "WorkerPingResult";
        const FIELDS: &'static [&'static str] = &["poweroff", "task"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                poweroff: self::de::required(&mut fields, "poweroff")?,
                task: self::de::defaulted(&mut fields, "task")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for WorkerPingResult {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl WorkerPingResult {
        pub fn builder() -> builder::WorkerPingResult {
            ::std::default::Default::default()
        }
    }

    ///`WorkerPingTask`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "output_rules",
    ///    "script"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "output_rules": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "script": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct WorkerPingTask {
        pub id: ::std::string::String,
        pub output_rules: ::std::vec::Vec<::std::string::String>,
        pub script: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for WorkerPingTask {
        const NAME: &'static str = "WorkerPingTask";
        const FIELDS: &'static [&'static str] = &["id", "output_rules", "script"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                id: self::de::required(&mut fields, "id")?,
                output_rules: self::de::required(&mut fields, "output_rules")?,
                script: self::de::required(&mut fields, "script")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for WorkerPingTask {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl WorkerPingTask {
        pub fn builder() -> builder::WorkerPingTask {
            ::std::default::Default::default()
        }
    }

    ///`WorkerTask`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "name",
    ///    "owner"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "owner": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct WorkerTask {
        pub id: ::std::string::String,
        pub name: ::std::string::String,
        pub owner: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for WorkerTask {
        const NAME: &'static str = "WorkerTask";
        const FIELDS: &'static [&'static str] = &["id", "name", "owner"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                id: self::de::required(&mut fields, "id")?,
                name: self::de::required(&mut fields, "name")?,
                owner: self::de::required(&mut fields, "owner")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for WorkerTask {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl WorkerTask {
        pub fn builder() -> builder::WorkerTask {
            ::std::default::Default::default()
        }
    }

    ///`WorkersResult`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "workers"
    ///  ],
    ///  "properties": {
    ///    "workers": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Worker"
    ///      }
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Debug, schemars :: JsonSchema)]
    pub struct WorkersResult {
        pub workers: ::std::vec::Vec<Worker>,
    }

    impl<'de> self::de::Build<'de> for WorkersResult {
        const NAME: &'static str = "WorkersResult";
        const FIELDS: &'static [&'static str] = &["workers"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                workers: self::de::required(&mut fields, "workers")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for WorkersResult {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl WorkersResult {
        pub fn builder() -> builder::WorkersResult {
            ::std::default::Default::default()
        }
    }

    /// Types for composing complex structures.
    pub mod builder {
        #[derive(Clone, Debug)]
        pub struct ObjWithOptionArray {
            stranger_things: ::std::result::Result<
                ::std::vec::Vec<::std::option::Option<super::Task>>,
                ::std::string::String,
            >,
            things: ::std::result::Result<
                ::std::vec::Vec<::std::option::Option<super::Task>>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for ObjWithOptionArray {
            fn default() -> Self {
                Self {
                    stranger_things: Err("no value supplied for stranger_things".to_string()),
                    things: Err("no value supplied for things".to_string()),
                }
            }
        }

        impl ObjWithOptionArray {
            pub fn stranger_things<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<::std::option::Option<super::Task>>>,
                T::Error: ::std::fmt::Display,
            {
                self.stranger_things = value.try_into().map_err(|e| {
                    format!("error converting supplied value for stranger_things: {e}")
                });
                self
            }
            pub fn things<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<::std::option::Option<super::Task>>>,
                T::Error: ::std::fmt::Display,
            {
                self.things = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for things: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<ObjWithOptionArray> for super::ObjWithOptionArray {
            type Error = super::error::ConversionError;
            fn try_from(
                value: ObjWithOptionArray,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    stranger_things: value.stranger_things?,
                    things: value.things?,
                })
            }
        }

        impl ::std::convert::From<super::ObjWithOptionArray> for ObjWithOptionArray {
            fn from(value: super::ObjWithOptionArray) -> Self {
                Self {
                    stranger_things: Ok(value.stranger_things),
                    things: Ok(value.things),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Task {
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            output_rules: ::std::result::Result<
                ::std::vec::Vec<::std::string::String>,
                ::std::string::String,
            >,
            script: ::std::result::Result<::std::string::String, ::std::string::String>,
            state: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for Task {
            fn default() -> Self {
                Self {
                    id: Err("no value supplied for id".to_string()),
                    name: Err("no value supplied for name".to_string()),
                    output_rules: Err("no value supplied for output_rules".to_string()),
                    script: Err("no value supplied for script".to_string()),
                    state: Err("no value supplied for state".to_string()),
                }
            }
        }

        impl Task {
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {e}"));
                self
            }
            pub fn output_rules<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.output_rules = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for output_rules: {e}"));
                self
            }
            pub fn script<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.script = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for script: {e}"));
                self
            }
            pub fn state<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.state = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for state: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<Task> for super::Task {
            type Error = super::error::ConversionError;
            fn try_from(value: Task) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    id: value.id?,
                    name: value.name?,
                    output_rules: value.output_rules?,
                    script: value.script?,
                    state: value.state?,
                })
            }
        }

        impl ::std::convert::From<super::Task> for Task {
            fn from(value: super::Task) -> Self {
                Self {
                    id: Ok(value.id),
                    name: Ok(value.name),
                    output_rules: Ok(value.output_rules),
                    script: Ok(value.script),
                    state: Ok(value.state),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct TaskEvent {
            payload: ::std::result::Result<::std::string::String, ::std::string::String>,
            seq: ::std::result::Result<u32, ::std::string::String>,
            stream: ::std::result::Result<::std::string::String, ::std::string::String>,
            time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for TaskEvent {
            fn default() -> Self {
                Self {
                    payload: Err("no value supplied for payload".to_string()),
                    seq: Err("no value supplied for seq".to_string()),
                    stream: Err("no value supplied for stream".to_string()),
                    time: Err("no value supplied for time".to_string()),
                }
            }
        }

        impl TaskEvent {
            pub fn payload<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.payload = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for payload: {e}"));
                self
            }
            pub fn seq<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u32>,
                T::Error: ::std::fmt::Display,
            {
                self.seq = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for seq: {e}"));
                self
            }
            pub fn stream<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.stream = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for stream: {e}"));
                self
            }
            pub fn time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for time: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<TaskEvent> for super::TaskEvent {
            type Error = super::error::ConversionError;
            fn try_from(
                value: TaskEvent,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    payload: value.payload?,
                    seq: value.seq?,
                    stream: value.stream?,
                    time: value.time?,
                })
            }
        }

        impl ::std::convert::From<super::TaskEvent> for TaskEvent {
            fn from(value: super::TaskEvent) -> Self {
                Self {
                    payload: Ok(value.payload),
                    seq: Ok(value.seq),
                    stream: Ok(value.stream),
                    time: Ok(value.time),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct TaskOutput {
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
            path: ::std::result::Result<::std::string::String, ::std::string::String>,
            size: ::std::result::Result<u64, ::std::string::String>,
        }

        impl ::std::default::Default for TaskOutput {
            fn default() -> Self {
                Self {
                    id: Err("no value supplied for id".to_string()),
                    path: Err("no value supplied for path".to_string()),
                    size: Err("no value supplied for size".to_string()),
                }
            }
        }

        impl TaskOutput {
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
            pub fn path<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.path = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for path: {e}"));
                self
            }
            pub fn size<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u64>,
                T::Error: ::std::fmt::Display,
            {
                self.size = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for size: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<TaskOutput> for super::TaskOutput {
            type Error = super::error::ConversionError;
            fn try_from(
                value: TaskOutput,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    id: value.id?,
                    path: value.path?,
                    size: value.size?,
                })
            }
        }

        impl ::std::convert::From<super::TaskOutput> for TaskOutput {
            fn from(value: super::TaskOutput) -> Self {
                Self {
                    id: Ok(value.id),
                    path: Ok(value.path),
                    size: Ok(value.size),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct TaskSubmit {
            default: ::std::result::Result<bool, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            output_rules: ::std::result::Result<
                ::std::vec::Vec<::std::string::String>,
                ::std::string::String,
            >,
            script: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for TaskSubmit {
            fn default() -> Self {
                Self {
                    default: Err("no value supplied for default".to_string()),
                    name: Err("no value supplied for name".to_string()),
                    output_rules: Ok(::std::default::Default::default()),
                    script: Err("no value supplied for script".to_string()),
                }
            }
        }

        impl TaskSubmit {
            pub fn default<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.default = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for default: {e}"));
                self
            }
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {e}"));
                self
            }
            pub fn output_rules<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.output_rules = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for output_rules: {e}"));
                self
            }
            pub fn script<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.script = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for script: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<TaskSubmit> for super::TaskSubmit {
            type Error = super::error::ConversionError;
            fn try_from(
                value: TaskSubmit,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    default: value.default?,
                    name: value.name?,
                    output_rules: value.output_rules?,
                    script: value.script?,
                })
            }
        }

        impl ::std::convert::From<super::TaskSubmit> for TaskSubmit {
            fn from(value: super::TaskSubmit) -> Self {
                Self {
                    default: Ok(value.default),
                    name: Ok(value.name),
                    output_rules: Ok(value.output_rules),
                    script: Ok(value.script),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct TaskSubmitResult {
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for TaskSubmitResult {
            fn default() -> Self {
                Self {
                    id: Err("no value supplied for id".to_string()),
                }
            }
        }

        impl TaskSubmitResult {
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<TaskSubmitResult> for super::TaskSubmitResult {
            type Error = super::error::ConversionError;
            fn try_from(
                value: TaskSubmitResult,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self { id: value.id? })
            }
        }

        impl ::std::convert::From<super::TaskSubmitResult> for TaskSubmitResult {
            fn from(value: super::TaskSubmitResult) -> Self {
                Self { id: Ok(value.id) }
            }
        }

        pub use self::TaskSubmitResult as UploadedChunk;
        #[derive(Clone, Debug)]
        pub struct UserCreate {
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for UserCreate {
            fn default() -> Self {
                Self {
                    name: Err("no value supplied for name".to_string()),
                }
            }
        }

        impl UserCreate {
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<UserCreate> for super::UserCreate {
            type Error = super::error::ConversionError;
            fn try_from(
                value: UserCreate,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self { name: value.name? })
            }
        }

        impl ::std::convert::From<super::UserCreate> for UserCreate {
            fn from(value: super::UserCreate) -> Self {
                Self {
                    name: Ok(value.name),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct UserCreateResult {
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            token: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for UserCreateResult {
            fn default() -> Self {
                Self {
                    id: Err("no value supplied for id".to_string()),
                    name: Err("no value supplied for name".to_string()),
                    token: Err("no value supplied for token".to_string()),
                }
            }
        }

        impl UserCreateResult {
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {e}"));
                self
            }
            pub fn token<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.token = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for token: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<UserCreateResult> for super::UserCreateResult {
            type Error = super::error::ConversionError;
            fn try_from(
                value: UserCreateResult,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    id: value.id?,
                    name: value.name?,
                    token: value.token?,
                })
            }
        }

        impl ::std::convert::From<super::UserCreateResult> for UserCreateResult {
            fn from(value: super::UserCreateResult) -> Self {
                Self {
                    id: Ok(value.id),
                    name: Ok(value.name),
                    token: Ok(value.token),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct WhoamiResult {
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for WhoamiResult {
            fn default() -> Self {
                Self {
                    id: Err("no value supplied for id".to_string()),
                    name: Err("no value supplied for name".to_string()),
                }
            }
        }

        impl WhoamiResult {
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<WhoamiResult> for super::WhoamiResult {
            type Error = super::error::ConversionError;
            fn try_from(
                value: WhoamiResult,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    id: value.id?,
                    name: value.name?,
                })
            }
        }

        impl ::std::convert::From<super::WhoamiResult> for WhoamiResult {
            fn from(value: super::WhoamiResult) -> Self {
                Self {
                    id: Ok(value.id),
                    name: Ok(value.name),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Worker {
            deleted: ::std::result::Result<bool, ::std::string::String>,
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
            instance_id: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            lastping: ::std::result::Result<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                ::std::string::String,
            >,
            recycle: ::std::result::Result<bool, ::std::string::String>,
            tasks: ::std::result::Result<::std::vec::Vec<super::WorkerTask>, ::std::string::String>,
        }

        impl ::std::default::Default for Worker {
            fn default() -> Self {
                Self {
                    deleted: Err("no value supplied for deleted".to_string()),
                    id: Err("no value supplied for id".to_string()),
                    instance_id: Ok(::std::default::Default::default()),
                    lastping: Ok(::std::default::Default::default()),
                    recycle: Err("no value supplied for recycle".to_string()),
                    tasks: Err("no value supplied for tasks".to_string()),
                }
            }
        }

        impl Worker {
            pub fn deleted<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.deleted = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for deleted: {e}"));
                self
            }
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
            pub fn instance_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.instance_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for instance_id: {e}"));
                self
            }
            pub fn lastping<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                    ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                >,
                T::Error: ::std::fmt::Display,
            {
                self.lastping = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for lastping: {e}"));
                self
            }
            pub fn recycle<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.recycle = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for recycle: {e}"));
                self
            }
            pub fn tasks<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::WorkerTask>>,
                T::Error: ::std::fmt::Display,
            {
                self.tasks = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for tasks: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<Worker> for super::Worker {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Worker,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    deleted: value.deleted?,
                    id: value.id?,
                    instance_id: value.instance_id?,
                    lastping: value.lastping?,
                    recycle: value.recycle?,
                    tasks: value.tasks?,
                })
            }
        }

        impl ::std::convert::From<super::Worker> for Worker {
            fn from(value: super::Worker) -> Self {
                Self {
                    deleted: Ok(value.deleted),
                    id: Ok(value.id),
                    instance_id: Ok(value.instance_id),
                    lastping: Ok(value.lastping),
                    recycle: Ok(value.recycle),
                    tasks: Ok(value.tasks),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct WorkerAddOutput {
            chunks: ::std::result::Result<
                ::std::vec::Vec<::std::string::String>,
                ::std::string::String,
            >,
            path: ::std::result::Result<::std::string::String, ::std::string::String>,
            size: ::std::result::Result<i64, ::std::string::String>,
        }

        impl ::std::default::Default for WorkerAddOutput {
            fn default() -> Self {
                Self {
                    chunks: Err("no value supplied for chunks".to_string()),
                    path: Err("no value supplied for path".to_string()),
                    size: Err("no value supplied for size".to_string()),
                }
            }
        }

        impl WorkerAddOutput {
            pub fn chunks<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.chunks = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for chunks: {e}"));
                self
            }
            pub fn path<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.path = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for path: {e}"));
                self
            }
            pub fn size<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i64>,
                T::Error: ::std::fmt::Display,
            {
                self.size = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for size: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<WorkerAddOutput> for super::WorkerAddOutput {
            type Error = super::error::ConversionError;
            fn try_from(
                value: WorkerAddOutput,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    chunks: value.chunks?,
                    path: value.path?,
                    size: value.size?,
                })
            }
        }

        impl ::std::convert::From<super::WorkerAddOutput> for WorkerAddOutput {
            fn from(value: super::WorkerAddOutput) -> Self {
                Self {
                    chunks: Ok(value.chunks),
                    path: Ok(value.path),
                    size: Ok(value.size),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct WorkerAppendTask {
            payload: ::std::result::Result<::std::string::String, ::std::string::String>,
            stream: ::std::result::Result<::std::string::String, ::std::string::String>,
            time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for WorkerAppendTask {
            fn default() -> Self {
                Self {
                    payload: Err("no value supplied for payload".to_string()),
                    stream: Err("no value supplied for stream".to_string()),
                    time: Err("no value supplied for time".to_string()),
                }
            }
        }

        impl WorkerAppendTask {
            pub fn payload<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.payload = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for payload: {e}"));
                self
            }
            pub fn stream<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.stream = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for stream: {e}"));
                self
            }
            pub fn time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for time: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<WorkerAppendTask> for super::WorkerAppendTask {
            type Error = super::error::ConversionError;
            fn try_from(
                value: WorkerAppendTask,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    payload: value.payload?,
                    stream: value.stream?,
                    time: value.time?,
                })
            }
        }

        impl ::std::convert::From<super::WorkerAppendTask> for WorkerAppendTask {
            fn from(value: super::WorkerAppendTask) -> Self {
                Self {
                    payload: Ok(value.payload),
                    stream: Ok(value.stream),
                    time: Ok(value.time),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct WorkerBootstrap {
            bootstrap: ::std::result::Result<::std::string::String, ::std::string::String>,
            token: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for WorkerBootstrap {
            fn default() -> Self {
                Self {
                    bootstrap: Err("no value supplied for bootstrap".to_string()),
                    token: Err("no value supplied for token".to_string()),
                }
            }
        }

        impl WorkerBootstrap {
            pub fn bootstrap<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.bootstrap = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for bootstrap: {e}"));
                self
            }
            pub fn token<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.token = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for token: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<WorkerBootstrap> for super::WorkerBootstrap {
            type Error = super::error::ConversionError;
            fn try_from(
                value: WorkerBootstrap,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    bootstrap: value.bootstrap?,
                    token: value.token?,
                })
            }
        }

        impl ::std::convert::From<super::WorkerBootstrap> for WorkerBootstrap {
            fn from(value: super::WorkerBootstrap) -> Self {
                Self {
                    bootstrap: Ok(value.bootstrap),
                    token: Ok(value.token),
                }
            }
        }

        pub use self::TaskSubmitResult as WorkerBootstrapResult;
        #[derive(Clone, Debug)]
        pub struct WorkerCompleteTask {
            failed: ::std::result::Result<bool, ::std::string::String>,
        }

        impl ::std::default::Default for WorkerCompleteTask {
            fn default() -> Self {
                Self {
                    failed: Err("no value supplied for failed".to_string()),
                }
            }
        }

        impl WorkerCompleteTask {
            pub fn failed<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.failed = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for failed: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<WorkerCompleteTask> for super::WorkerCompleteTask {
            type Error = super::error::ConversionError;
            fn try_from(
                value: WorkerCompleteTask,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    failed: value.failed?,
                })
            }
        }

        impl ::std::convert::From<super::WorkerCompleteTask> for WorkerCompleteTask {
            fn from(value: super::WorkerCompleteTask) -> Self {
                Self {
                    failed: Ok(value.failed),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct WorkerPingResult {
            poweroff: ::std::result::Result<bool, ::std::string::String>,
            task: ::std::result::Result<
                ::std::option::Option<super::WorkerPingTask>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for WorkerPingResult {
            fn default() -> Self {
                Self {
                    poweroff: Err("no value supplied for poweroff".to_string()),
                    task: Ok(::std::default::Default::default()),
                }
            }
        }

        impl WorkerPingResult {
            pub fn poweroff<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.poweroff = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for poweroff: {e}"));
                self
            }
            pub fn task<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::WorkerPingTask>>,
                T::Error: ::std::fmt::Display,
            {
                self.task = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for task: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<WorkerPingResult> for super::WorkerPingResult {
            type Error = super::error::ConversionError;
            fn try_from(
                value: WorkerPingResult,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    poweroff: value.poweroff?,
                    task: value.task?,
                })
            }
        }

        impl ::std::convert::From<super::WorkerPingResult> for WorkerPingResult {
            fn from(value: super::WorkerPingResult) -> Self {
                Self {
                    poweroff: Ok(value.poweroff),
                    task: Ok(value.task),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct WorkerPingTask {
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
            output_rules: ::std::result::Result<
                ::std::vec::Vec<::std::string::String>,
                ::std::string::String,
            >,
            script: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for WorkerPingTask {
            fn default() -> Self {
                Self {
                    id: Err("no value supplied for id".to_string()),
                    output_rules: Err("no value supplied for output_rules".to_string()),
                    script: Err("no value supplied for script".to_string()),
                }
            }
        }

        impl WorkerPingTask {
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
            pub fn output_rules<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.output_rules = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for output_rules: {e}"));
                self
            }
            pub fn script<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.script = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for script: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<WorkerPingTask> for super::WorkerPingTask {
            type Error = super::error::ConversionError;
            fn try_from(
                value: WorkerPingTask,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    id: value.id?,
                    output_rules: value.output_rules?,
                    script: value.script?,
                })
            }
        }

        impl ::std::convert::From<super::WorkerPingTask> for WorkerPingTask {
            fn from(value: super::WorkerPingTask) -> Self {
                Self {
                    id: Ok(value.id),
                    output_rules: Ok(value.output_rules),
                    script: Ok(value.script),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct WorkerTask {
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            owner: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for WorkerTask {
            fn default() -> Self {
                Self {
                    id: Err("no value supplied for id".to_string()),
                    name: Err("no value supplied for name".to_string()),
                    owner: Err("no value supplied for owner".to_string()),
                }
            }
        }

        impl WorkerTask {
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {e}"));
                self
            }
            pub fn owner<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.owner = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for owner: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<WorkerTask> for super::WorkerTask {
            type Error = super::error::ConversionError;
            fn try_from(
                value: WorkerTask,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    id: value.id?,
                    name: value.name?,
                    owner: value.owner?,
                })
            }
        }

        impl ::std::convert::From<super::WorkerTask> for WorkerTask {
            fn from(value: super::WorkerTask) -> Self {
                Self {
                    id: Ok(value.id),
                    name: Ok(value.name),
                    owner: Ok(value.owner),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct WorkersResult {
            workers: ::std::result::Result<::std::vec::Vec<super::Worker>, ::std::string::String>,
        }

        impl ::std::default::Default for WorkersResult {
            fn default() -> Self {
                Self {
                    workers: Err("no value supplied for workers".to_string()),
                }
            }
        }

        impl WorkersResult {
            pub fn workers<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Worker>>,
                T::Error: ::std::fmt::Display,
            {
                self.workers = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for workers: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<WorkersResult> for super::WorkersResult {
            type Error = super::error::ConversionError;
            fn try_from(
                value: WorkersResult,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    workers: value.workers?,
                })
            }
        }

        impl ::std::convert::From<super::WorkersResult> for WorkersResult {
            fn from(value: super::WorkersResult) -> Self {
                Self {
                    workers: Ok(value.workers),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
///Client for Buildomat
///
///Version: 1.0
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}

impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = ::std::time::Duration::from_secs(15u64);
            reqwest::ClientBuilder::new()
                .connect_timeout(dur)
                .timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }

    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }

    /// Run the request through the pre/post hooks and
    /// [`ClientHooks::exec`], yielding the raw response.
    #[doc(hidden)]
    #[allow(dead_code, clippy::all)]
    pub(crate) async fn __progenitor_dispatch<E>(
        &self,
        #[allow(unused_mut)] mut request: ::reqwest::Request,
        info: &OperationInfo,
    ) -> ::std::result::Result<::reqwest::Response, Error<E>> {
        self.pre(&mut request, info).await?;
        let result = self.exec(request, info).await;
        self.post(&result, info).await?;
        ::std::result::Result::Ok(result?)
    }

    /// Execute the request and decode the response for the common
    /// shape: one JSON success status, an optional JSON error status
    /// or range, and anything else unexpected.
    ///
    /// Operations matching that shape call this instead of inlining
    /// their own `match`, which is worth doing for the same reason as
    /// [`Self::__progenitor_dispatch`]: the two
    /// `ResponseValue::from_response` calls are `async`, so inlined
    /// they cost two more opaque future types per operation.
    /// `status_arm_pattern`/`success_arm_pattern` decide eligibility —
    /// the `if`/`else if` order below reproduces match-arm precedence,
    /// which is success-before-error-before-catch-all.
    #[doc(hidden)]
    #[allow(dead_code, clippy::all)]
    pub(crate) async fn __progenitor_response<T, E>(
        &self,
        request: ::reqwest::Request,
        info: &OperationInfo,
        success: &[(u16, u16)],
        error: &[(u16, u16)],
    ) -> ::std::result::Result<ResponseValue<T>, Error<E>>
    where
        T: ::serde::de::DeserializeOwned,
        E: ::serde::de::DeserializeOwned,
    {
        let response = self.__progenitor_dispatch(request, info).await?;
        let status = response.status().as_u16();
        let matches_window = |windows: &[(u16, u16)]| {
            windows
                .iter()
                .any(|&(low, high)| status >= low && status <= high)
        };
        if matches_window(success) {
            ResponseValue::from_response(response).await
        } else if matches_window(error) {
            ::std::result::Result::Err(Error::ErrorResponse(
                ResponseValue::from_response(response).await?,
            ))
        } else {
            ::std::result::Result::Err(Error::UnexpectedResponse(response))
        }
    }
}

impl ClientInfo<()> for Client {
    fn api_version() -> &'static str {
        "1.0"
    }

    fn baseurl(&self) -> &str {
        self.baseurl.as_str()
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn inner(&self) -> &() {
        &()
    }
}

impl ClientHooks<()> for &Client {}
impl Client {
    ///Sends a `POST` request to `/v1/control/hold`
    ///
    ///```ignore
    /// let response = client.control_hold()
    ///    .send()
    ///    .await;
    /// ```
    pub fn control_hold(&self) -> builder::ControlHold<'_> {
        builder::ControlHold::new(self)
    }

    ///Sends a `POST` request to `/v1/control/resume`
    ///
    ///```ignore
    /// let response = client.control_resume()
    ///    .send()
    ///    .await;
    /// ```
    pub fn control_resume(&self) -> builder::ControlResume<'_> {
        builder::ControlResume::new(self)
    }

    ///Sends a `GET` request to `/v1/task/{Task}`
    ///
    ///```ignore
    /// let response = client.task_get()
    ///    .task(task)
    ///    .send()
    ///    .await;
    /// ```
    pub fn task_get(&self) -> builder::TaskGet<'_> {
        builder::TaskGet::new(self)
    }

    ///Sends a `GET` request to `/v1/tasks`
    ///
    ///```ignore
    /// let response = client.tasks_get()
    ///    .send()
    ///    .await;
    /// ```
    pub fn tasks_get(&self) -> builder::TasksGet<'_> {
        builder::TasksGet::new(self)
    }

    ///Sends a `POST` request to `/v1/tasks`
    ///
    ///```ignore
    /// let response = client.task_submit()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn task_submit(&self) -> builder::TaskSubmit<'_> {
        builder::TaskSubmit::new(self)
    }

    ///Sends a `GET` request to `/v1/tasks/{task}/events`
    ///
    ///```ignore
    /// let response = client.task_events_get()
    ///    .task(task)
    ///    .minseq(minseq)
    ///    .send()
    ///    .await;
    /// ```
    pub fn task_events_get(&self) -> builder::TaskEventsGet<'_> {
        builder::TaskEventsGet::new(self)
    }

    ///Sends a `GET` request to `/v1/tasks/{task}/outputs`
    ///
    ///```ignore
    /// let response = client.task_outputs_get()
    ///    .task(task)
    ///    .send()
    ///    .await;
    /// ```
    pub fn task_outputs_get(&self) -> builder::TaskOutputsGet<'_> {
        builder::TaskOutputsGet::new(self)
    }

    ///Sends a `GET` request to `/v1/tasks/{task}/outputs/{output}`
    ///
    ///```ignore
    /// let response = client.task_output_download()
    ///    .task(task)
    ///    .output(output)
    ///    .send()
    ///    .await;
    /// ```
    pub fn task_output_download(&self) -> builder::TaskOutputDownload<'_> {
        builder::TaskOutputDownload::new(self)
    }

    ///Sends a `POST` request to `/v1/users`
    ///
    ///```ignore
    /// let response = client.user_create()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn user_create(&self) -> builder::UserCreate<'_> {
        builder::UserCreate::new(self)
    }

    ///Sends a `GET` request to `/v1/whoami`
    ///
    ///```ignore
    /// let response = client.whoami()
    ///    .send()
    ///    .await;
    /// ```
    pub fn whoami(&self) -> builder::Whoami<'_> {
        builder::Whoami::new(self)
    }

    ///Sends a `PUT` request to `/v1/whoami/name`
    ///
    ///```ignore
    /// let response = client.whoami_put_name()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn whoami_put_name(&self) -> builder::WhoamiPutName<'_> {
        builder::WhoamiPutName::new(self)
    }

    ///Sends a `POST` request to `/v1/worker/bootstrap`
    ///
    ///```ignore
    /// let response = client.worker_bootstrap()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn worker_bootstrap(&self) -> builder::WorkerBootstrap<'_> {
        builder::WorkerBootstrap::new(self)
    }

    ///Sends a `GET` request to `/v1/worker/ping`
    ///
    ///```ignore
    /// let response = client.worker_ping()
    ///    .send()
    ///    .await;
    /// ```
    pub fn worker_ping(&self) -> builder::WorkerPing<'_> {
        builder::WorkerPing::new(self)
    }

    ///Sends a `POST` request to `/v1/worker/task/{task}/append`
    ///
    ///```ignore
    /// let response = client.worker_task_append()
    ///    .task(task)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn worker_task_append(&self) -> builder::WorkerTaskAppend<'_> {
        builder::WorkerTaskAppend::new(self)
    }

    ///Sends a `POST` request to `/v1/worker/task/{task}/chunk`
    ///
    ///```ignore
    /// let response = client.worker_task_upload_chunk()
    ///    .task(task)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn worker_task_upload_chunk(&self) -> builder::WorkerTaskUploadChunk<'_> {
        builder::WorkerTaskUploadChunk::new(self)
    }

    ///Sends a `POST` request to `/v1/worker/task/{task}/complete`
    ///
    ///```ignore
    /// let response = client.worker_task_complete()
    ///    .task(task)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn worker_task_complete(&self) -> builder::WorkerTaskComplete<'_> {
        builder::WorkerTaskComplete::new(self)
    }

    ///Sends a `POST` request to `/v1/worker/task/{task}/output`
    ///
    ///```ignore
    /// let response = client.worker_task_add_output()
    ///    .task(task)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn worker_task_add_output(&self) -> builder::WorkerTaskAddOutput<'_> {
        builder::WorkerTaskAddOutput::new(self)
    }

    ///Sends a `GET` request to `/v1/workers`
    ///
    ///```ignore
    /// let response = client.workers_list()
    ///    .send()
    ///    .await;
    /// ```
    pub fn workers_list(&self) -> builder::WorkersList<'_> {
        builder::WorkersList::new(self)
    }

    ///Sends a `POST` request to `/v1/workers/recycle`
    ///
    ///```ignore
    /// let response = client.workers_recycle()
    ///    .send()
    ///    .await;
    /// ```
    pub fn workers_recycle(&self) -> builder::WorkersRecycle<'_> {
        builder::WorkersRecycle::new(self)
    }

    ///Sends a `GET` request to `/v1/things`
    ///
    ///```ignore
    /// let response = client.get_thing_or_things()
    ///    .id(id)
    ///    .send()
    ///    .await;
    /// ```
    pub fn get_thing_or_things(&self) -> builder::GetThingOrThings<'_> {
        builder::GetThingOrThings::new(self)
    }

    ///Sends a `GET` request to `/v1/header-arg`
    ///
    ///```ignore
    /// let response = client.header_arg()
    ///    .accept_language(accept_language)
    ///    .send()
    ///    .await;
    /// ```
    pub fn header_arg(&self) -> builder::HeaderArg<'_> {
        builder::HeaderArg::new(self)
    }
}

/// Types for composing operation parameters.
#[allow(clippy::all)]
pub mod builder {
    use super::types;
    #[allow(unused_imports)]
    use super::{
        encode_path, ByteStream, ClientHooks, ClientInfo, Error, OperationInfo, RequestBuilderExt,
        ResponseValue,
    };
    ///Builder for [`Client::control_hold`]
    ///
    ///[`Client::control_hold`]: super::Client::control_hold
    #[derive(Debug, Clone)]
    pub struct ControlHold<'a> {
        client: &'a super::Client,
    }

    impl<'a> ControlHold<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `POST` request to `/v1/control/hold`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/v1/control/hold", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "control_hold",
            };
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::control_resume`]
    ///
    ///[`Client::control_resume`]: super::Client::control_resume
    #[derive(Debug, Clone)]
    pub struct ControlResume<'a> {
        client: &'a super::Client,
    }

    impl<'a> ControlResume<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `POST` request to `/v1/control/resume`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/v1/control/resume", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.post(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "control_resume",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::task_get`]
    ///
    ///[`Client::task_get`]: super::Client::task_get
    #[derive(Debug, Clone)]
    pub struct TaskGet<'a> {
        client: &'a super::Client,
        task: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> TaskGet<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                task: Err("task was not initialized".to_string()),
            }
        }

        pub fn task<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.task = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for task failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/v1/task/{Task}`
        pub async fn send(self) -> Result<ResponseValue<types::Task>, Error<()>> {
            let Self { client, task } = self;
            let task = task.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/v1/task/{}",
                client.baseurl,
                encode_path(&task.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "task_get",
            };
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::tasks_get`]
    ///
    ///[`Client::tasks_get`]: super::Client::tasks_get
    #[derive(Debug, Clone)]
    pub struct TasksGet<'a> {
        client: &'a super::Client,
    }

    impl<'a> TasksGet<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/v1/tasks`
        pub async fn send(self) -> Result<ResponseValue<::std::vec::Vec<types::Task>>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/v1/tasks", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "tasks_get",
            };
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::task_submit`]
    ///
    ///[`Client::task_submit`]: super::Client::task_submit
    #[derive(Debug, Clone)]
    pub struct TaskSubmit<'a> {
        client: &'a super::Client,
        body: ::std::result::Result<types::builder::TaskSubmit, ::std::string::String>,
    }

    impl<'a> TaskSubmit<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::TaskSubmit>,
            <V as std::convert::TryInto<types::TaskSubmit>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `TaskSubmit` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::TaskSubmit) -> types::builder::TaskSubmit,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/v1/tasks`
        pub async fn send(self) -> Result<ResponseValue<types::TaskSubmitResult>, Error<()>> {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::TaskSubmit::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/v1/tasks", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "task_submit",
            };
            client
                .__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::task_events_get`]
    ///
    ///[`Client::task_events_get`]: super::Client::task_events_get
    #[derive(Debug, Clone)]
    pub struct TaskEventsGet<'a> {
        client: &'a super::Client,
        task: ::std::result::Result<::std::string::String, ::std::string::String>,
        minseq: ::std::result::Result<::std::option::Option<u32>, ::std::string::String>,
    }

    impl<'a> TaskEventsGet<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                task: Err("task was not initialized".to_string()),
                minseq: Ok(None),
            }
        }

        pub fn task<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.task = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for task failed".to_string()
            });
            self
        }

        pub fn minseq<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<u32>,
        {
            self.minseq = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `u32` for minseq failed".to_string());
            self
        }

        ///Sends a `GET` request to `/v1/tasks/{task}/events`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<::std::vec::Vec<types::TaskEvent>>, Error<()>> {
            let Self {
                client,
                task,
                minseq,
            } = self;
            let task = task.map_err(Error::InvalidRequest)?;
            let minseq = minseq.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/v1/tasks/{}/events",
                client.baseurl,
                encode_path(&task.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("minseq", &minseq))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "task_events_get",
            };
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::task_outputs_get`]
    ///
    ///[`Client::task_outputs_get`]: super::Client::task_outputs_get
    #[derive(Debug, Clone)]
    pub struct TaskOutputsGet<'a> {
        client: &'a super::Client,
        task: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> TaskOutputsGet<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                task: Err("task was not initialized".to_string()),
            }
        }

        pub fn task<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.task = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for task failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/v1/tasks/{task}/outputs`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<::std::vec::Vec<types::TaskOutput>>, Error<()>> {
            let Self { client, task } = self;
            let task = task.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/v1/tasks/{}/outputs",
                client.baseurl,
                encode_path(&task.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "task_outputs_get",
            };
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::task_output_download`]
    ///
    ///[`Client::task_output_download`]: super::Client::task_output_download
    #[derive(Debug, Clone)]
    pub struct TaskOutputDownload<'a> {
        client: &'a super::Client,
        task: ::std::result::Result<::std::string::String, ::std::string::String>,
        output: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> TaskOutputDownload<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                task: Err("task was not initialized".to_string()),
                output: Err("output was not initialized".to_string()),
            }
        }

        pub fn task<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.task = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for task failed".to_string()
            });
            self
        }

        pub fn output<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.output = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for output failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/v1/tasks/{task}/outputs/{output}`
        pub async fn send(self) -> Result<ResponseValue<ByteStream>, Error<()>> {
            let Self {
                client,
                task,
                output,
            } = self;
            let task = task.map_err(Error::InvalidRequest)?;
            let output = output.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/v1/tasks/{}/outputs/{}",
                client.baseurl,
                encode_path(&task.to_string()),
                encode_path(&output.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.get(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "task_output_download",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200..=299 => Ok(ResponseValue::stream(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::user_create`]
    ///
    ///[`Client::user_create`]: super::Client::user_create
    #[derive(Debug, Clone)]
    pub struct UserCreate<'a> {
        client: &'a super::Client,
        body: ::std::result::Result<types::builder::UserCreate, ::std::string::String>,
    }

    impl<'a> UserCreate<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::UserCreate>,
            <V as std::convert::TryInto<types::UserCreate>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `UserCreate` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::UserCreate) -> types::builder::UserCreate,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/v1/users`
        pub async fn send(self) -> Result<ResponseValue<types::UserCreateResult>, Error<()>> {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::UserCreate::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/v1/users", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "user_create",
            };
            client
                .__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::whoami`]
    ///
    ///[`Client::whoami`]: super::Client::whoami
    #[derive(Debug, Clone)]
    pub struct Whoami<'a> {
        client: &'a super::Client,
    }

    impl<'a> Whoami<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/v1/whoami`
        pub async fn send(self) -> Result<ResponseValue<types::WhoamiResult>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/v1/whoami", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "whoami",
            };
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::whoami_put_name`]
    ///
    ///[`Client::whoami_put_name`]: super::Client::whoami_put_name
    #[derive(Debug)]
    pub struct WhoamiPutName<'a> {
        client: &'a super::Client,
        body: ::std::result::Result<reqwest::Body, ::std::string::String>,
    }

    impl<'a> WhoamiPutName<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Err("body was not initialized".to_string()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<String>,
        {
            self.body = value
                .try_into()
                .map_err(|_| "conversion to `String` for body failed".to_string())
                .map(|v| v.into());
            self
        }

        ///Sends a `PUT` request to `/v1/whoami/name`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client, body } = self;
            let body = body.map_err(Error::InvalidRequest)?;
            let url = format!("{}/v1/whoami/name", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .put(url)
                .header(
                    ::reqwest::header::CONTENT_TYPE,
                    ::reqwest::header::HeaderValue::from_static("text/plain"),
                )
                .body(body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "whoami_put_name",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::worker_bootstrap`]
    ///
    ///[`Client::worker_bootstrap`]: super::Client::worker_bootstrap
    #[derive(Debug, Clone)]
    pub struct WorkerBootstrap<'a> {
        client: &'a super::Client,
        body: ::std::result::Result<types::builder::WorkerBootstrap, ::std::string::String>,
    }

    impl<'a> WorkerBootstrap<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::WorkerBootstrap>,
            <V as std::convert::TryInto<types::WorkerBootstrap>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `WorkerBootstrap` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::WorkerBootstrap) -> types::builder::WorkerBootstrap,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/v1/worker/bootstrap`
        pub async fn send(self) -> Result<ResponseValue<types::WorkerBootstrapResult>, Error<()>> {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::WorkerBootstrap::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/v1/worker/bootstrap", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "worker_bootstrap",
            };
            client
                .__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::worker_ping`]
    ///
    ///[`Client::worker_ping`]: super::Client::worker_ping
    #[derive(Debug, Clone)]
    pub struct WorkerPing<'a> {
        client: &'a super::Client,
    }

    impl<'a> WorkerPing<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/v1/worker/ping`
        pub async fn send(self) -> Result<ResponseValue<types::WorkerPingResult>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/v1/worker/ping", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "worker_ping",
            };
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::worker_task_append`]
    ///
    ///[`Client::worker_task_append`]: super::Client::worker_task_append
    #[derive(Debug, Clone)]
    pub struct WorkerTaskAppend<'a> {
        client: &'a super::Client,
        task: ::std::result::Result<::std::string::String, ::std::string::String>,
        body: ::std::result::Result<types::builder::WorkerAppendTask, ::std::string::String>,
    }

    impl<'a> WorkerTaskAppend<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                task: Err("task was not initialized".to_string()),
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn task<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.task = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for task failed".to_string()
            });
            self
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::WorkerAppendTask>,
            <V as std::convert::TryInto<types::WorkerAppendTask>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `WorkerAppendTask` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(
                types::builder::WorkerAppendTask,
            ) -> types::builder::WorkerAppendTask,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/v1/worker/task/{task}/append`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client, task, body } = self;
            let task = task.map_err(Error::InvalidRequest)?;
            let body = body
                .and_then(|v| types::WorkerAppendTask::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/v1/worker/task/{}/append",
                client.baseurl,
                encode_path(&task.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "worker_task_append",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                201u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::worker_task_upload_chunk`]
    ///
    ///[`Client::worker_task_upload_chunk`]: super::Client::worker_task_upload_chunk
    #[derive(Debug)]
    pub struct WorkerTaskUploadChunk<'a> {
        client: &'a super::Client,
        task: ::std::result::Result<::std::string::String, ::std::string::String>,
        body: ::std::result::Result<reqwest::Body, ::std::string::String>,
    }

    impl<'a> WorkerTaskUploadChunk<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                task: Err("task was not initialized".to_string()),
                body: Err("body was not initialized".to_string()),
            }
        }

        pub fn task<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.task = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for task failed".to_string()
            });
            self
        }

        pub fn body<B>(mut self, value: B) -> Self
        where
            B: std::convert::TryInto<reqwest::Body>,
        {
            self.body = value
                .try_into()
                .map_err(|_| "conversion to `reqwest::Body` for body failed".to_string());
            self
        }

        ///Sends a `POST` request to `/v1/worker/task/{task}/chunk`
        pub async fn send(self) -> Result<ResponseValue<types::UploadedChunk>, Error<()>> {
            let Self { client, task, body } = self;
            let task = task.map_err(Error::InvalidRequest)?;
            let body = body.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/v1/worker/task/{}/chunk",
                client.baseurl,
                encode_path(&task.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .header(
                    ::reqwest::header::CONTENT_TYPE,
                    ::reqwest::header::HeaderValue::from_static("application/octet-stream"),
                )
                .body(body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "worker_task_upload_chunk",
            };
            client
                .__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::worker_task_complete`]
    ///
    ///[`Client::worker_task_complete`]: super::Client::worker_task_complete
    #[derive(Debug, Clone)]
    pub struct WorkerTaskComplete<'a> {
        client: &'a super::Client,
        task: ::std::result::Result<::std::string::String, ::std::string::String>,
        body: ::std::result::Result<types::builder::WorkerCompleteTask, ::std::string::String>,
    }

    impl<'a> WorkerTaskComplete<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                task: Err("task was not initialized".to_string()),
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn task<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.task = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for task failed".to_string()
            });
            self
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::WorkerCompleteTask>,
            <V as std::convert::TryInto<types::WorkerCompleteTask>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `WorkerCompleteTask` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(
                types::builder::WorkerCompleteTask,
            ) -> types::builder::WorkerCompleteTask,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/v1/worker/task/{task}/complete`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client, task, body } = self;
            let task = task.map_err(Error::InvalidRequest)?;
            let body = body
                .and_then(|v| types::WorkerCompleteTask::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/v1/worker/task/{}/complete",
                client.baseurl,
                encode_path(&task.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "worker_task_complete",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::worker_task_add_output`]
    ///
    ///[`Client::worker_task_add_output`]: super::Client::worker_task_add_output
    #[derive(Debug, Clone)]
    pub struct WorkerTaskAddOutput<'a> {
        client: &'a super::Client,
        task: ::std::result::Result<::std::string::String, ::std::string::String>,
        body: ::std::result::Result<types::builder::WorkerAddOutput, ::std::string::String>,
    }

    impl<'a> WorkerTaskAddOutput<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                task: Err("task was not initialized".to_string()),
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn task<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.task = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for task failed".to_string()
            });
            self
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::WorkerAddOutput>,
            <V as std::convert::TryInto<types::WorkerAddOutput>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `WorkerAddOutput` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::WorkerAddOutput) -> types::builder::WorkerAddOutput,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/v1/worker/task/{task}/output`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client, task, body } = self;
            let task = task.map_err(Error::InvalidRequest)?;
            let body = body
                .and_then(|v| types::WorkerAddOutput::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/v1/worker/task/{}/output",
                client.baseurl,
                encode_path(&task.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "worker_task_add_output",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                201u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::workers_list`]
    ///
    ///[`Client::workers_list`]: super::Client::workers_list
    #[derive(Debug, Clone)]
    pub struct WorkersList<'a> {
        client: &'a super::Client,
    }

    impl<'a> WorkersList<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/v1/workers`
        pub async fn send(self) -> Result<ResponseValue<types::WorkersResult>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/v1/workers", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "workers_list",
            };
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::workers_recycle`]
    ///
    ///[`Client::workers_recycle`]: super::Client::workers_recycle
    #[derive(Debug, Clone)]
    pub struct WorkersRecycle<'a> {
        client: &'a super::Client,
    }

    impl<'a> WorkersRecycle<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `POST` request to `/v1/workers/recycle`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/v1/workers/recycle", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.post(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "workers_recycle",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::get_thing_or_things`]
    ///
    ///[`Client::get_thing_or_things`]: super::Client::get_thing_or_things
    #[derive(Debug, Clone)]
    pub struct GetThingOrThings<'a> {
        client: &'a super::Client,
        id: ::std::result::Result<
            ::std::option::Option<types::GetThingOrThingsId>,
            ::std::string::String,
        >,
    }

    impl<'a> GetThingOrThings<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                id: Ok(None),
            }
        }

        pub fn id<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::GetThingOrThingsId>,
        {
            self.id = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `GetThingOrThingsId` for id failed".to_string());
            self
        }

        ///Sends a `GET` request to `/v1/things`
        pub async fn send(self) -> Result<ResponseValue<::std::string::String>, Error<()>> {
            let Self { client, id } = self;
            let id = id.map_err(Error::InvalidRequest)?;
            let url = format!("{}/v1/things", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("id", &id))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "get_thing_or_things",
            };
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::header_arg`]
    ///
    ///[`Client::header_arg`]: super::Client::header_arg
    #[derive(Debug, Clone)]
    pub struct HeaderArg<'a> {
        client: &'a super::Client,
        accept_language: ::std::result::Result<
            ::std::option::Option<types::HeaderArgAcceptLanguage>,
            ::std::string::String,
        >,
    }

    impl<'a> HeaderArg<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                accept_language: Ok(None),
            }
        }

        pub fn accept_language<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::HeaderArgAcceptLanguage>,
        {
            self.accept_language = value.try_into().map(Some).map_err(|_| {
                "conversion to `HeaderArgAcceptLanguage` for accept_language failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/v1/header-arg`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self {
                client,
                accept_language,
            } = self;
            let accept_language = accept_language.map_err(Error::InvalidRequest)?;
            let url = format!("{}/v1/header-arg", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            if let Some(value) = accept_language {
                header_map.append("accept-language", value.to_string().try_into()?);
            }
            #[allow(unused_mut)]
            let mut request = client.client.get(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "header_arg",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200..=299 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            }
        }
    }
}

/// Items consumers will typically use such as the Client.
pub mod prelude {
    pub use self::super::Client;
}
