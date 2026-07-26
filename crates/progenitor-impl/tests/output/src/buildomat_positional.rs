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
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
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
    #[derive(:: serde :: Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    #[derive(Clone, Debug)]
    pub struct ObjWithOptionArray {
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

    impl ::serde::Serialize for ObjWithOptionArray {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "ObjWithOptionArray", len)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "stranger-things",
                &self.stranger_things,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "things", &self.things)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for Task {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1 + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "Task", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "output_rules",
                &self.output_rules,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "script", &self.script)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "state", &self.state)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for TaskEvent {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "TaskEvent", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "payload", &self.payload)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "seq", &self.seq)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "stream", &self.stream)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "time", &self.time)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for TaskOutput {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "TaskOutput", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "path", &self.path)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "size", &self.size)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
    pub struct TaskSubmit {
        pub default: bool,
        pub name: ::std::string::String,
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

    impl ::serde::Serialize for TaskSubmit {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize
                + 1
                + 1
                + ::std::primitive::usize::from(!::std::vec::Vec::is_empty(&self.output_rules))
                + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "TaskSubmit", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "default", &self.default)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
            if !::std::vec::Vec::is_empty(&self.output_rules) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "output_rules",
                    &self.output_rules,
                )?;
            }
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "script", &self.script)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for TaskSubmitResult {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "TaskSubmitResult", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for UserCreate {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "UserCreate", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for UserCreateResult {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "UserCreateResult", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "token", &self.token)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for WhoamiResult {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "WhoamiResult", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
    pub struct Worker {
        pub deleted: bool,
        pub id: ::std::string::String,
        pub instance_id: ::std::option::Option<::std::string::String>,
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

    impl ::serde::Serialize for Worker {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize
                + 1
                + 1
                + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.instance_id))
                + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.lastping))
                + 1
                + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "Worker", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "deleted", &self.deleted)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            if !::std::option::Option::is_none(&self.instance_id) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "instance_id",
                    &self.instance_id,
                )?;
            }
            if !::std::option::Option::is_none(&self.lastping) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "lastping",
                    &self.lastping,
                )?;
            }
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "recycle", &self.recycle)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "tasks", &self.tasks)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for WorkerAddOutput {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "WorkerAddOutput", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "chunks", &self.chunks)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "path", &self.path)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "size", &self.size)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for WorkerAppendTask {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "WorkerAppendTask", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "payload", &self.payload)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "stream", &self.stream)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "time", &self.time)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for WorkerBootstrap {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "WorkerBootstrap", len)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "bootstrap",
                &self.bootstrap,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "token", &self.token)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for WorkerCompleteTask {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "WorkerCompleteTask", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "failed", &self.failed)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
    pub struct WorkerPingResult {
        pub poweroff: bool,
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

    impl ::serde::Serialize for WorkerPingResult {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize
                + 1
                + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.task));
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "WorkerPingResult", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "poweroff", &self.poweroff)?;
            if !::std::option::Option::is_none(&self.task) {
                ::serde::ser::SerializeStruct::serialize_field(&mut state, "task", &self.task)?;
            }
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for WorkerPingTask {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "WorkerPingTask", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "output_rules",
                &self.output_rules,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "script", &self.script)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for WorkerTask {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "WorkerTask", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "owner", &self.owner)?;
            ::serde::ser::SerializeStruct::end(state)
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
    #[derive(Clone, Debug)]
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

    impl ::serde::Serialize for WorkersResult {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "WorkersResult", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "workers", &self.workers)?;
            ::serde::ser::SerializeStruct::end(state)
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
#[allow(clippy::all)]
impl Client {
    ///Sends a `POST` request to `/v1/control/hold`
    pub async fn control_hold<'a>(&'a self) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/v1/control/hold", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
            .await
    }

    ///Sends a `POST` request to `/v1/control/resume`
    pub async fn control_resume<'a>(&'a self) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/v1/control/resume", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.post(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "control_resume",
        };
        let response = self.__progenitor_dispatch(request, &info).await?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `GET` request to `/v1/task/{Task}`
    pub async fn task_get<'a>(
        &'a self,
        task: &'a str,
    ) -> Result<ResponseValue<types::Task>, Error<()>> {
        let url = format!(
            "{}/v1/task/{}",
            self.baseurl,
            encode_path(&task.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
            .await
    }

    ///Sends a `GET` request to `/v1/tasks`
    pub async fn tasks_get<'a>(
        &'a self,
    ) -> Result<ResponseValue<::std::vec::Vec<types::Task>>, Error<()>> {
        let url = format!("{}/v1/tasks", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
            .await
    }

    ///Sends a `POST` request to `/v1/tasks`
    pub async fn task_submit<'a>(
        &'a self,
        body: &'a types::TaskSubmit,
    ) -> Result<ResponseValue<types::TaskSubmitResult>, Error<()>> {
        let url = format!("{}/v1/tasks", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
            .await
    }

    ///Sends a `GET` request to `/v1/tasks/{task}/events`
    pub async fn task_events_get<'a>(
        &'a self,
        task: &'a str,
        minseq: ::std::option::Option<u32>,
    ) -> Result<ResponseValue<::std::vec::Vec<types::TaskEvent>>, Error<()>> {
        let url = format!(
            "{}/v1/tasks/{}/events",
            self.baseurl,
            encode_path(&task.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
            .await
    }

    ///Sends a `GET` request to `/v1/tasks/{task}/outputs`
    pub async fn task_outputs_get<'a>(
        &'a self,
        task: &'a str,
    ) -> Result<ResponseValue<::std::vec::Vec<types::TaskOutput>>, Error<()>> {
        let url = format!(
            "{}/v1/tasks/{}/outputs",
            self.baseurl,
            encode_path(&task.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
            .await
    }

    ///Sends a `GET` request to `/v1/tasks/{task}/outputs/{output}`
    pub async fn task_output_download<'a>(
        &'a self,
        task: &'a str,
        output: &'a str,
    ) -> Result<ResponseValue<ByteStream>, Error<()>> {
        let url = format!(
            "{}/v1/tasks/{}/outputs/{}",
            self.baseurl,
            encode_path(&task.to_string()),
            encode_path(&output.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "task_output_download",
        };
        let response = self.__progenitor_dispatch(request, &info).await?;
        match response.status().as_u16() {
            200..=299 => Ok(ResponseValue::stream(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `POST` request to `/v1/users`
    pub async fn user_create<'a>(
        &'a self,
        body: &'a types::UserCreate,
    ) -> Result<ResponseValue<types::UserCreateResult>, Error<()>> {
        let url = format!("{}/v1/users", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
            .await
    }

    ///Sends a `GET` request to `/v1/whoami`
    pub async fn whoami<'a>(&'a self) -> Result<ResponseValue<types::WhoamiResult>, Error<()>> {
        let url = format!("{}/v1/whoami", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
            .await
    }

    ///Sends a `PUT` request to `/v1/whoami/name`
    pub async fn whoami_put_name<'a>(
        &'a self,
        body: ::std::string::String,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/v1/whoami/name", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        let response = self.__progenitor_dispatch(request, &info).await?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `POST` request to `/v1/worker/bootstrap`
    pub async fn worker_bootstrap<'a>(
        &'a self,
        body: &'a types::WorkerBootstrap,
    ) -> Result<ResponseValue<types::WorkerBootstrapResult>, Error<()>> {
        let url = format!("{}/v1/worker/bootstrap", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
            .await
    }

    ///Sends a `GET` request to `/v1/worker/ping`
    pub async fn worker_ping<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::WorkerPingResult>, Error<()>> {
        let url = format!("{}/v1/worker/ping", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
            .await
    }

    ///Sends a `POST` request to `/v1/worker/task/{task}/append`
    pub async fn worker_task_append<'a>(
        &'a self,
        task: &'a str,
        body: &'a types::WorkerAppendTask,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!(
            "{}/v1/worker/task/{}/append",
            self.baseurl,
            encode_path(&task.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .json(&body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "worker_task_append",
        };
        let response = self.__progenitor_dispatch(request, &info).await?;
        match response.status().as_u16() {
            201u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `POST` request to `/v1/worker/task/{task}/chunk`
    pub async fn worker_task_upload_chunk<'a, B: Into<reqwest::Body>>(
        &'a self,
        task: &'a str,
        body: B,
    ) -> Result<ResponseValue<types::UploadedChunk>, Error<()>> {
        let url = format!(
            "{}/v1/worker/task/{}/chunk",
            self.baseurl,
            encode_path(&task.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
            .await
    }

    ///Sends a `POST` request to `/v1/worker/task/{task}/complete`
    pub async fn worker_task_complete<'a>(
        &'a self,
        task: &'a str,
        body: &'a types::WorkerCompleteTask,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!(
            "{}/v1/worker/task/{}/complete",
            self.baseurl,
            encode_path(&task.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .json(&body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "worker_task_complete",
        };
        let response = self.__progenitor_dispatch(request, &info).await?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `POST` request to `/v1/worker/task/{task}/output`
    pub async fn worker_task_add_output<'a>(
        &'a self,
        task: &'a str,
        body: &'a types::WorkerAddOutput,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!(
            "{}/v1/worker/task/{}/output",
            self.baseurl,
            encode_path(&task.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .json(&body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "worker_task_add_output",
        };
        let response = self.__progenitor_dispatch(request, &info).await?;
        match response.status().as_u16() {
            201u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `GET` request to `/v1/workers`
    pub async fn workers_list<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::WorkersResult>, Error<()>> {
        let url = format!("{}/v1/workers", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
            .await
    }

    ///Sends a `POST` request to `/v1/workers/recycle`
    pub async fn workers_recycle<'a>(&'a self) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/v1/workers/recycle", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.post(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "workers_recycle",
        };
        let response = self.__progenitor_dispatch(request, &info).await?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `GET` request to `/v1/things`
    pub async fn get_thing_or_things<'a>(
        &'a self,
        id: ::std::option::Option<&'a types::GetThingOrThingsId>,
    ) -> Result<ResponseValue<::std::string::String>, Error<()>> {
        let url = format!("{}/v1/things", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self
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
        self.__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
            .await
    }

    ///Sends a `GET` request to `/v1/header-arg`
    pub async fn header_arg<'a>(
        &'a self,
        accept_language: ::std::option::Option<types::HeaderArgAcceptLanguage>,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/v1/header-arg", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        if let Some(value) = accept_language {
            header_map.append("accept-language", value.to_string().try_into()?);
        }

        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "header_arg",
        };
        let response = self.__progenitor_dispatch(request, &info).await?;
        match response.status().as_u16() {
            200..=299 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::ErrorResponse(ResponseValue::empty(response))),
        }
    }
}

/// Items consumers will typically use such as the Client.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
}
