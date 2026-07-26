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

    ///`EnrolBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "title": "EnrolBody",
    ///  "type": "object",
    ///  "required": [
    ///    "host",
    ///    "key"
    ///  ],
    ///  "properties": {
    ///    "host": {
    ///      "type": "string"
    ///    },
    ///    "key": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct EnrolBody {
        pub host: ::std::string::String,
        pub key: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for EnrolBody {
        const NAME: &'static str = "EnrolBody";
        const FIELDS: &'static [&'static str] = &["host", "key"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                host: self::de::required(&mut fields, "host")?,
                key: self::de::required(&mut fields, "key")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for EnrolBody {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for EnrolBody {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "EnrolBody", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "host", &self.host)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "key", &self.key)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl EnrolBody {
        pub fn builder() -> builder::EnrolBody {
            ::std::default::Default::default()
        }
    }

    ///`GlobalJobsResult`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "title": "GlobalJobsResult",
    ///  "type": "object",
    ///  "required": [
    ///    "summary"
    ///  ],
    ///  "properties": {
    ///    "summary": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ReportSummary"
    ///      }
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct GlobalJobsResult {
        pub summary: ::std::vec::Vec<ReportSummary>,
    }

    impl<'de> self::de::Build<'de> for GlobalJobsResult {
        const NAME: &'static str = "GlobalJobsResult";
        const FIELDS: &'static [&'static str] = &["summary"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                summary: self::de::required(&mut fields, "summary")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for GlobalJobsResult {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for GlobalJobsResult {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "GlobalJobsResult", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "summary", &self.summary)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl GlobalJobsResult {
        pub fn builder() -> builder::GlobalJobsResult {
            ::std::default::Default::default()
        }
    }

    ///`OutputRecord`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "msg",
    ///    "stream",
    ///    "time"
    ///  ],
    ///  "properties": {
    ///    "msg": {
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
    pub struct OutputRecord {
        pub msg: ::std::string::String,
        pub stream: ::std::string::String,
        pub time: ::chrono::DateTime<::chrono::offset::Utc>,
    }

    impl<'de> self::de::Build<'de> for OutputRecord {
        const NAME: &'static str = "OutputRecord";
        const FIELDS: &'static [&'static str] = &["msg", "stream", "time"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                msg: self::de::required(&mut fields, "msg")?,
                stream: self::de::required(&mut fields, "stream")?,
                time: self::de::required(&mut fields, "time")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for OutputRecord {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for OutputRecord {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "OutputRecord", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "msg", &self.msg)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "stream", &self.stream)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "time", &self.time)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl OutputRecord {
        pub fn builder() -> builder::OutputRecord {
            ::std::default::Default::default()
        }
    }

    ///`PingResult`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "title": "PingResult",
    ///  "type": "object",
    ///  "required": [
    ///    "host",
    ///    "ok"
    ///  ],
    ///  "properties": {
    ///    "host": {
    ///      "type": "string"
    ///    },
    ///    "ok": {
    ///      "type": "boolean"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct PingResult {
        pub host: ::std::string::String,
        pub ok: bool,
    }

    impl<'de> self::de::Build<'de> for PingResult {
        const NAME: &'static str = "PingResult";
        const FIELDS: &'static [&'static str] = &["host", "ok"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                host: self::de::required(&mut fields, "host")?,
                ok: self::de::required(&mut fields, "ok")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for PingResult {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for PingResult {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "PingResult", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "host", &self.host)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "ok", &self.ok)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl PingResult {
        pub fn builder() -> builder::PingResult {
            ::std::default::Default::default()
        }
    }

    ///`ReportFinishBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "title": "ReportFinishBody",
    ///  "type": "object",
    ///  "required": [
    ///    "duration_millis",
    ///    "end_time",
    ///    "exit_status",
    ///    "id"
    ///  ],
    ///  "properties": {
    ///    "duration_millis": {
    ///      "type": "integer",
    ///      "format": "int32"
    ///    },
    ///    "end_time": {
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "exit_status": {
    ///      "type": "integer",
    ///      "format": "int32"
    ///    },
    ///    "id": {
    ///      "$ref": "#/components/schemas/ReportId"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct ReportFinishBody {
        pub duration_millis: i32,
        pub end_time: ::chrono::DateTime<::chrono::offset::Utc>,
        pub exit_status: i32,
        pub id: ReportId,
    }

    impl<'de> self::de::Build<'de> for ReportFinishBody {
        const NAME: &'static str = "ReportFinishBody";
        const FIELDS: &'static [&'static str] =
            &["duration_millis", "end_time", "exit_status", "id"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                duration_millis: self::de::required(&mut fields, "duration_millis")?,
                end_time: self::de::required(&mut fields, "end_time")?,
                exit_status: self::de::required(&mut fields, "exit_status")?,
                id: self::de::required(&mut fields, "id")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for ReportFinishBody {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for ReportFinishBody {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "ReportFinishBody", len)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "duration_millis",
                &self.duration_millis,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "end_time", &self.end_time)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "exit_status",
                &self.exit_status,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl ReportFinishBody {
        pub fn builder() -> builder::ReportFinishBody {
            ::std::default::Default::default()
        }
    }

    ///`ReportId`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "host",
    ///    "job",
    ///    "pid",
    ///    "time",
    ///    "uuid"
    ///  ],
    ///  "properties": {
    ///    "host": {
    ///      "type": "string"
    ///    },
    ///    "job": {
    ///      "type": "string"
    ///    },
    ///    "pid": {
    ///      "type": "integer",
    ///      "format": "uint64",
    ///      "minimum": 0.0
    ///    },
    ///    "time": {
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "uuid": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct ReportId {
        pub host: ::std::string::String,
        pub job: ::std::string::String,
        pub pid: u64,
        pub time: ::chrono::DateTime<::chrono::offset::Utc>,
        pub uuid: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for ReportId {
        const NAME: &'static str = "ReportId";
        const FIELDS: &'static [&'static str] = &["host", "job", "pid", "time", "uuid"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                host: self::de::required(&mut fields, "host")?,
                job: self::de::required(&mut fields, "job")?,
                pid: self::de::required(&mut fields, "pid")?,
                time: self::de::required(&mut fields, "time")?,
                uuid: self::de::required(&mut fields, "uuid")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for ReportId {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for ReportId {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1 + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "ReportId", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "host", &self.host)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "job", &self.job)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "pid", &self.pid)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "time", &self.time)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "uuid", &self.uuid)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl ReportId {
        pub fn builder() -> builder::ReportId {
            ::std::default::Default::default()
        }
    }

    ///`ReportOutputBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "title": "ReportOutputBody",
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "record"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "$ref": "#/components/schemas/ReportId"
    ///    },
    ///    "record": {
    ///      "$ref": "#/components/schemas/OutputRecord"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct ReportOutputBody {
        pub id: ReportId,
        pub record: OutputRecord,
    }

    impl<'de> self::de::Build<'de> for ReportOutputBody {
        const NAME: &'static str = "ReportOutputBody";
        const FIELDS: &'static [&'static str] = &["id", "record"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                id: self::de::required(&mut fields, "id")?,
                record: self::de::required(&mut fields, "record")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for ReportOutputBody {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for ReportOutputBody {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "ReportOutputBody", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "record", &self.record)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl ReportOutputBody {
        pub fn builder() -> builder::ReportOutputBody {
            ::std::default::Default::default()
        }
    }

    ///`ReportResult`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "title": "ReportResult",
    ///  "type": "object",
    ///  "required": [
    ///    "existed_already"
    ///  ],
    ///  "properties": {
    ///    "existed_already": {
    ///      "type": "boolean"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct ReportResult {
        pub existed_already: bool,
    }

    impl<'de> self::de::Build<'de> for ReportResult {
        const NAME: &'static str = "ReportResult";
        const FIELDS: &'static [&'static str] = &["existed_already"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                existed_already: self::de::required(&mut fields, "existed_already")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for ReportResult {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for ReportResult {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "ReportResult", len)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "existed_already",
                &self.existed_already,
            )?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl ReportResult {
        pub fn builder() -> builder::ReportResult {
            ::std::default::Default::default()
        }
    }

    ///`ReportStartBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "title": "ReportStartBody",
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "script",
    ///    "start_time"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "$ref": "#/components/schemas/ReportId"
    ///    },
    ///    "script": {
    ///      "type": "string"
    ///    },
    ///    "start_time": {
    ///      "type": "string",
    ///      "format": "date-time"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct ReportStartBody {
        pub id: ReportId,
        pub script: ::std::string::String,
        pub start_time: ::chrono::DateTime<::chrono::offset::Utc>,
    }

    impl<'de> self::de::Build<'de> for ReportStartBody {
        const NAME: &'static str = "ReportStartBody";
        const FIELDS: &'static [&'static str] = &["id", "script", "start_time"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                id: self::de::required(&mut fields, "id")?,
                script: self::de::required(&mut fields, "script")?,
                start_time: self::de::required(&mut fields, "start_time")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for ReportStartBody {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for ReportStartBody {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "ReportStartBody", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "script", &self.script)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "start_time",
                &self.start_time,
            )?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl ReportStartBody {
        pub fn builder() -> builder::ReportStartBody {
            ::std::default::Default::default()
        }
    }

    ///`ReportSummary`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "age_seconds",
    ///    "duration_seconds",
    ///    "host",
    ///    "job",
    ///    "status",
    ///    "when"
    ///  ],
    ///  "properties": {
    ///    "age_seconds": {
    ///      "type": "integer",
    ///      "format": "int32"
    ///    },
    ///    "duration_seconds": {
    ///      "type": "integer",
    ///      "format": "int32"
    ///    },
    ///    "host": {
    ///      "type": "string"
    ///    },
    ///    "job": {
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "type": "integer",
    ///      "format": "int32"
    ///    },
    ///    "when": {
    ///      "type": "string",
    ///      "format": "date-time"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct ReportSummary {
        pub age_seconds: i32,
        pub duration_seconds: i32,
        pub host: ::std::string::String,
        pub job: ::std::string::String,
        pub status: i32,
        pub when: ::chrono::DateTime<::chrono::offset::Utc>,
    }

    impl<'de> self::de::Build<'de> for ReportSummary {
        const NAME: &'static str = "ReportSummary";
        const FIELDS: &'static [&'static str] = &[
            "age_seconds",
            "duration_seconds",
            "host",
            "job",
            "status",
            "when",
        ];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                age_seconds: self::de::required(&mut fields, "age_seconds")?,
                duration_seconds: self::de::required(&mut fields, "duration_seconds")?,
                host: self::de::required(&mut fields, "host")?,
                job: self::de::required(&mut fields, "job")?,
                status: self::de::required(&mut fields, "status")?,
                when: self::de::required(&mut fields, "when")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for ReportSummary {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for ReportSummary {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1 + 1 + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "ReportSummary", len)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "age_seconds",
                &self.age_seconds,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "duration_seconds",
                &self.duration_seconds,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "host", &self.host)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "job", &self.job)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "status", &self.status)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "when", &self.when)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl ReportSummary {
        pub fn builder() -> builder::ReportSummary {
            ::std::default::Default::default()
        }
    }

    /// Types for composing complex structures.
    pub mod builder {
        #[derive(Clone, Debug)]
        pub struct EnrolBody {
            host: ::std::result::Result<::std::string::String, ::std::string::String>,
            key: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for EnrolBody {
            fn default() -> Self {
                Self {
                    host: Err("no value supplied for host".to_string()),
                    key: Err("no value supplied for key".to_string()),
                }
            }
        }

        impl EnrolBody {
            pub fn host<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.host = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for host: {e}"));
                self
            }
            pub fn key<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.key = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for key: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<EnrolBody> for super::EnrolBody {
            type Error = super::error::ConversionError;
            fn try_from(
                value: EnrolBody,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    host: value.host?,
                    key: value.key?,
                })
            }
        }

        impl ::std::convert::From<super::EnrolBody> for EnrolBody {
            fn from(value: super::EnrolBody) -> Self {
                Self {
                    host: Ok(value.host),
                    key: Ok(value.key),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct GlobalJobsResult {
            summary:
                ::std::result::Result<::std::vec::Vec<super::ReportSummary>, ::std::string::String>,
        }

        impl ::std::default::Default for GlobalJobsResult {
            fn default() -> Self {
                Self {
                    summary: Err("no value supplied for summary".to_string()),
                }
            }
        }

        impl GlobalJobsResult {
            pub fn summary<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::ReportSummary>>,
                T::Error: ::std::fmt::Display,
            {
                self.summary = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for summary: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<GlobalJobsResult> for super::GlobalJobsResult {
            type Error = super::error::ConversionError;
            fn try_from(
                value: GlobalJobsResult,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    summary: value.summary?,
                })
            }
        }

        impl ::std::convert::From<super::GlobalJobsResult> for GlobalJobsResult {
            fn from(value: super::GlobalJobsResult) -> Self {
                Self {
                    summary: Ok(value.summary),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct OutputRecord {
            msg: ::std::result::Result<::std::string::String, ::std::string::String>,
            stream: ::std::result::Result<::std::string::String, ::std::string::String>,
            time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for OutputRecord {
            fn default() -> Self {
                Self {
                    msg: Err("no value supplied for msg".to_string()),
                    stream: Err("no value supplied for stream".to_string()),
                    time: Err("no value supplied for time".to_string()),
                }
            }
        }

        impl OutputRecord {
            pub fn msg<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.msg = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for msg: {e}"));
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

        impl ::std::convert::TryFrom<OutputRecord> for super::OutputRecord {
            type Error = super::error::ConversionError;
            fn try_from(
                value: OutputRecord,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    msg: value.msg?,
                    stream: value.stream?,
                    time: value.time?,
                })
            }
        }

        impl ::std::convert::From<super::OutputRecord> for OutputRecord {
            fn from(value: super::OutputRecord) -> Self {
                Self {
                    msg: Ok(value.msg),
                    stream: Ok(value.stream),
                    time: Ok(value.time),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct PingResult {
            host: ::std::result::Result<::std::string::String, ::std::string::String>,
            ok: ::std::result::Result<bool, ::std::string::String>,
        }

        impl ::std::default::Default for PingResult {
            fn default() -> Self {
                Self {
                    host: Err("no value supplied for host".to_string()),
                    ok: Err("no value supplied for ok".to_string()),
                }
            }
        }

        impl PingResult {
            pub fn host<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.host = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for host: {e}"));
                self
            }
            pub fn ok<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.ok = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for ok: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<PingResult> for super::PingResult {
            type Error = super::error::ConversionError;
            fn try_from(
                value: PingResult,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    host: value.host?,
                    ok: value.ok?,
                })
            }
        }

        impl ::std::convert::From<super::PingResult> for PingResult {
            fn from(value: super::PingResult) -> Self {
                Self {
                    host: Ok(value.host),
                    ok: Ok(value.ok),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct ReportFinishBody {
            duration_millis: ::std::result::Result<i32, ::std::string::String>,
            end_time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            exit_status: ::std::result::Result<i32, ::std::string::String>,
            id: ::std::result::Result<super::ReportId, ::std::string::String>,
        }

        impl ::std::default::Default for ReportFinishBody {
            fn default() -> Self {
                Self {
                    duration_millis: Err("no value supplied for duration_millis".to_string()),
                    end_time: Err("no value supplied for end_time".to_string()),
                    exit_status: Err("no value supplied for exit_status".to_string()),
                    id: Err("no value supplied for id".to_string()),
                }
            }
        }

        impl ReportFinishBody {
            pub fn duration_millis<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i32>,
                T::Error: ::std::fmt::Display,
            {
                self.duration_millis = value.try_into().map_err(|e| {
                    format!("error converting supplied value for duration_millis: {e}")
                });
                self
            }
            pub fn end_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.end_time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for end_time: {e}"));
                self
            }
            pub fn exit_status<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i32>,
                T::Error: ::std::fmt::Display,
            {
                self.exit_status = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for exit_status: {e}"));
                self
            }
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::ReportId>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<ReportFinishBody> for super::ReportFinishBody {
            type Error = super::error::ConversionError;
            fn try_from(
                value: ReportFinishBody,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    duration_millis: value.duration_millis?,
                    end_time: value.end_time?,
                    exit_status: value.exit_status?,
                    id: value.id?,
                })
            }
        }

        impl ::std::convert::From<super::ReportFinishBody> for ReportFinishBody {
            fn from(value: super::ReportFinishBody) -> Self {
                Self {
                    duration_millis: Ok(value.duration_millis),
                    end_time: Ok(value.end_time),
                    exit_status: Ok(value.exit_status),
                    id: Ok(value.id),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct ReportId {
            host: ::std::result::Result<::std::string::String, ::std::string::String>,
            job: ::std::result::Result<::std::string::String, ::std::string::String>,
            pid: ::std::result::Result<u64, ::std::string::String>,
            time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            uuid: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for ReportId {
            fn default() -> Self {
                Self {
                    host: Err("no value supplied for host".to_string()),
                    job: Err("no value supplied for job".to_string()),
                    pid: Err("no value supplied for pid".to_string()),
                    time: Err("no value supplied for time".to_string()),
                    uuid: Err("no value supplied for uuid".to_string()),
                }
            }
        }

        impl ReportId {
            pub fn host<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.host = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for host: {e}"));
                self
            }
            pub fn job<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.job = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for job: {e}"));
                self
            }
            pub fn pid<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u64>,
                T::Error: ::std::fmt::Display,
            {
                self.pid = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for pid: {e}"));
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
            pub fn uuid<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.uuid = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for uuid: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<ReportId> for super::ReportId {
            type Error = super::error::ConversionError;
            fn try_from(
                value: ReportId,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    host: value.host?,
                    job: value.job?,
                    pid: value.pid?,
                    time: value.time?,
                    uuid: value.uuid?,
                })
            }
        }

        impl ::std::convert::From<super::ReportId> for ReportId {
            fn from(value: super::ReportId) -> Self {
                Self {
                    host: Ok(value.host),
                    job: Ok(value.job),
                    pid: Ok(value.pid),
                    time: Ok(value.time),
                    uuid: Ok(value.uuid),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct ReportOutputBody {
            id: ::std::result::Result<super::ReportId, ::std::string::String>,
            record: ::std::result::Result<super::OutputRecord, ::std::string::String>,
        }

        impl ::std::default::Default for ReportOutputBody {
            fn default() -> Self {
                Self {
                    id: Err("no value supplied for id".to_string()),
                    record: Err("no value supplied for record".to_string()),
                }
            }
        }

        impl ReportOutputBody {
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::ReportId>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
            pub fn record<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::OutputRecord>,
                T::Error: ::std::fmt::Display,
            {
                self.record = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for record: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<ReportOutputBody> for super::ReportOutputBody {
            type Error = super::error::ConversionError;
            fn try_from(
                value: ReportOutputBody,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    id: value.id?,
                    record: value.record?,
                })
            }
        }

        impl ::std::convert::From<super::ReportOutputBody> for ReportOutputBody {
            fn from(value: super::ReportOutputBody) -> Self {
                Self {
                    id: Ok(value.id),
                    record: Ok(value.record),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct ReportResult {
            existed_already: ::std::result::Result<bool, ::std::string::String>,
        }

        impl ::std::default::Default for ReportResult {
            fn default() -> Self {
                Self {
                    existed_already: Err("no value supplied for existed_already".to_string()),
                }
            }
        }

        impl ReportResult {
            pub fn existed_already<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.existed_already = value.try_into().map_err(|e| {
                    format!("error converting supplied value for existed_already: {e}")
                });
                self
            }
        }

        impl ::std::convert::TryFrom<ReportResult> for super::ReportResult {
            type Error = super::error::ConversionError;
            fn try_from(
                value: ReportResult,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    existed_already: value.existed_already?,
                })
            }
        }

        impl ::std::convert::From<super::ReportResult> for ReportResult {
            fn from(value: super::ReportResult) -> Self {
                Self {
                    existed_already: Ok(value.existed_already),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct ReportStartBody {
            id: ::std::result::Result<super::ReportId, ::std::string::String>,
            script: ::std::result::Result<::std::string::String, ::std::string::String>,
            start_time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for ReportStartBody {
            fn default() -> Self {
                Self {
                    id: Err("no value supplied for id".to_string()),
                    script: Err("no value supplied for script".to_string()),
                    start_time: Err("no value supplied for start_time".to_string()),
                }
            }
        }

        impl ReportStartBody {
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::ReportId>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
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
            pub fn start_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.start_time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for start_time: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<ReportStartBody> for super::ReportStartBody {
            type Error = super::error::ConversionError;
            fn try_from(
                value: ReportStartBody,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    id: value.id?,
                    script: value.script?,
                    start_time: value.start_time?,
                })
            }
        }

        impl ::std::convert::From<super::ReportStartBody> for ReportStartBody {
            fn from(value: super::ReportStartBody) -> Self {
                Self {
                    id: Ok(value.id),
                    script: Ok(value.script),
                    start_time: Ok(value.start_time),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct ReportSummary {
            age_seconds: ::std::result::Result<i32, ::std::string::String>,
            duration_seconds: ::std::result::Result<i32, ::std::string::String>,
            host: ::std::result::Result<::std::string::String, ::std::string::String>,
            job: ::std::result::Result<::std::string::String, ::std::string::String>,
            status: ::std::result::Result<i32, ::std::string::String>,
            when: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for ReportSummary {
            fn default() -> Self {
                Self {
                    age_seconds: Err("no value supplied for age_seconds".to_string()),
                    duration_seconds: Err("no value supplied for duration_seconds".to_string()),
                    host: Err("no value supplied for host".to_string()),
                    job: Err("no value supplied for job".to_string()),
                    status: Err("no value supplied for status".to_string()),
                    when: Err("no value supplied for when".to_string()),
                }
            }
        }

        impl ReportSummary {
            pub fn age_seconds<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i32>,
                T::Error: ::std::fmt::Display,
            {
                self.age_seconds = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for age_seconds: {e}"));
                self
            }
            pub fn duration_seconds<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i32>,
                T::Error: ::std::fmt::Display,
            {
                self.duration_seconds = value.try_into().map_err(|e| {
                    format!("error converting supplied value for duration_seconds: {e}")
                });
                self
            }
            pub fn host<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.host = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for host: {e}"));
                self
            }
            pub fn job<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.job = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for job: {e}"));
                self
            }
            pub fn status<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i32>,
                T::Error: ::std::fmt::Display,
            {
                self.status = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for status: {e}"));
                self
            }
            pub fn when<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.when = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for when: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<ReportSummary> for super::ReportSummary {
            type Error = super::error::ConversionError;
            fn try_from(
                value: ReportSummary,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    age_seconds: value.age_seconds?,
                    duration_seconds: value.duration_seconds?,
                    host: value.host?,
                    job: value.job?,
                    status: value.status?,
                    when: value.when?,
                })
            }
        }

        impl ::std::convert::From<super::ReportSummary> for ReportSummary {
            fn from(value: super::ReportSummary) -> Self {
                Self {
                    age_seconds: Ok(value.age_seconds),
                    duration_seconds: Ok(value.duration_seconds),
                    host: Ok(value.host),
                    job: Ok(value.job),
                    status: Ok(value.status),
                    when: Ok(value.when),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
///Client for Keeper API
///
///report execution of cron jobs through a mechanism other than mail
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
    ///Sends a `POST` request to `/enrol`
    ///
    ///Arguments:
    /// - `authorization`: Authorization header (bearer token)
    /// - `body`
    ///```ignore
    /// let response = client.enrol()
    ///    .authorization(authorization)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn enrol(&self) -> builder::Enrol<'_> {
        builder::Enrol::new(self)
    }

    ///Sends a `GET` request to `/global/jobs`
    ///
    ///Arguments:
    /// - `authorization`: Authorization header (bearer token)
    ///```ignore
    /// let response = client.global_jobs()
    ///    .authorization(authorization)
    ///    .send()
    ///    .await;
    /// ```
    pub fn global_jobs(&self) -> builder::GlobalJobs<'_> {
        builder::GlobalJobs::new(self)
    }

    ///Sends a `GET` request to `/ping`
    ///
    ///Arguments:
    /// - `authorization`: Authorization header (bearer token)
    ///```ignore
    /// let response = client.ping()
    ///    .authorization(authorization)
    ///    .send()
    ///    .await;
    /// ```
    pub fn ping(&self) -> builder::Ping<'_> {
        builder::Ping::new(self)
    }

    ///Sends a `POST` request to `/report/finish`
    ///
    ///Arguments:
    /// - `authorization`: Authorization header (bearer token)
    /// - `body`
    ///```ignore
    /// let response = client.report_finish()
    ///    .authorization(authorization)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn report_finish(&self) -> builder::ReportFinish<'_> {
        builder::ReportFinish::new(self)
    }

    ///Sends a `POST` request to `/report/output`
    ///
    ///Arguments:
    /// - `authorization`: Authorization header (bearer token)
    /// - `body`
    ///```ignore
    /// let response = client.report_output()
    ///    .authorization(authorization)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn report_output(&self) -> builder::ReportOutput<'_> {
        builder::ReportOutput::new(self)
    }

    ///Sends a `POST` request to `/report/start`
    ///
    ///Arguments:
    /// - `authorization`: Authorization header (bearer token)
    /// - `body`
    ///```ignore
    /// let response = client.report_start()
    ///    .authorization(authorization)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn report_start(&self) -> builder::ReportStart<'_> {
        builder::ReportStart::new(self)
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
    ///Builder for [`Client::enrol`]
    ///
    ///[`Client::enrol`]: super::Client::enrol
    #[derive(Debug, Clone)]
    pub struct Enrol<'a> {
        client: &'a super::Client,
        authorization: ::std::result::Result<::std::string::String, ::std::string::String>,
        body: ::std::result::Result<types::builder::EnrolBody, ::std::string::String>,
    }

    impl<'a> Enrol<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                authorization: Err("authorization was not initialized".to_string()),
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn authorization<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.authorization = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for authorization failed".to_string()
            });
            self
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::EnrolBody>,
            <V as std::convert::TryInto<types::EnrolBody>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `EnrolBody` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::EnrolBody) -> types::builder::EnrolBody,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/enrol`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self {
                client,
                authorization,
                body,
            } = self;
            let authorization = authorization.map_err(Error::InvalidRequest)?;
            let body = body
                .and_then(|v| types::EnrolBody::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/enrol", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            header_map.append("Authorization", authorization.to_string().try_into()?);
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "enrol",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                201u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::global_jobs`]
    ///
    ///[`Client::global_jobs`]: super::Client::global_jobs
    #[derive(Debug, Clone)]
    pub struct GlobalJobs<'a> {
        client: &'a super::Client,
        authorization: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> GlobalJobs<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                authorization: Err("authorization was not initialized".to_string()),
            }
        }

        pub fn authorization<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.authorization = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for authorization failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/global/jobs`
        pub async fn send(self) -> Result<ResponseValue<types::GlobalJobsResult>, Error<()>> {
            let Self {
                client,
                authorization,
            } = self;
            let authorization = authorization.map_err(Error::InvalidRequest)?;
            let url = format!("{}/global/jobs", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            header_map.append("Authorization", authorization.to_string().try_into()?);
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
                operation_id: "global_jobs",
            };
            client
                .__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::ping`]
    ///
    ///[`Client::ping`]: super::Client::ping
    #[derive(Debug, Clone)]
    pub struct Ping<'a> {
        client: &'a super::Client,
        authorization: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> Ping<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                authorization: Err("authorization was not initialized".to_string()),
            }
        }

        pub fn authorization<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.authorization = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for authorization failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/ping`
        pub async fn send(self) -> Result<ResponseValue<types::PingResult>, Error<()>> {
            let Self {
                client,
                authorization,
            } = self;
            let authorization = authorization.map_err(Error::InvalidRequest)?;
            let url = format!("{}/ping", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            header_map.append("Authorization", authorization.to_string().try_into()?);
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
                operation_id: "ping",
            };
            client
                .__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::report_finish`]
    ///
    ///[`Client::report_finish`]: super::Client::report_finish
    #[derive(Debug, Clone)]
    pub struct ReportFinish<'a> {
        client: &'a super::Client,
        authorization: ::std::result::Result<::std::string::String, ::std::string::String>,
        body: ::std::result::Result<types::builder::ReportFinishBody, ::std::string::String>,
    }

    impl<'a> ReportFinish<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                authorization: Err("authorization was not initialized".to_string()),
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn authorization<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.authorization = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for authorization failed".to_string()
            });
            self
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::ReportFinishBody>,
            <V as std::convert::TryInto<types::ReportFinishBody>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `ReportFinishBody` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(
                types::builder::ReportFinishBody,
            ) -> types::builder::ReportFinishBody,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/report/finish`
        pub async fn send(self) -> Result<ResponseValue<types::ReportResult>, Error<()>> {
            let Self {
                client,
                authorization,
                body,
            } = self;
            let authorization = authorization.map_err(Error::InvalidRequest)?;
            let body = body
                .and_then(|v| types::ReportFinishBody::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/report/finish", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            header_map.append("Authorization", authorization.to_string().try_into()?);
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
                operation_id: "report_finish",
            };
            client
                .__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::report_output`]
    ///
    ///[`Client::report_output`]: super::Client::report_output
    #[derive(Debug, Clone)]
    pub struct ReportOutput<'a> {
        client: &'a super::Client,
        authorization: ::std::result::Result<::std::string::String, ::std::string::String>,
        body: ::std::result::Result<types::builder::ReportOutputBody, ::std::string::String>,
    }

    impl<'a> ReportOutput<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                authorization: Err("authorization was not initialized".to_string()),
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn authorization<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.authorization = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for authorization failed".to_string()
            });
            self
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::ReportOutputBody>,
            <V as std::convert::TryInto<types::ReportOutputBody>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `ReportOutputBody` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(
                types::builder::ReportOutputBody,
            ) -> types::builder::ReportOutputBody,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/report/output`
        pub async fn send(self) -> Result<ResponseValue<types::ReportResult>, Error<()>> {
            let Self {
                client,
                authorization,
                body,
            } = self;
            let authorization = authorization.map_err(Error::InvalidRequest)?;
            let body = body
                .and_then(|v| types::ReportOutputBody::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/report/output", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            header_map.append("Authorization", authorization.to_string().try_into()?);
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
                operation_id: "report_output",
            };
            client
                .__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::report_start`]
    ///
    ///[`Client::report_start`]: super::Client::report_start
    #[derive(Debug, Clone)]
    pub struct ReportStart<'a> {
        client: &'a super::Client,
        authorization: ::std::result::Result<::std::string::String, ::std::string::String>,
        body: ::std::result::Result<types::builder::ReportStartBody, ::std::string::String>,
    }

    impl<'a> ReportStart<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                authorization: Err("authorization was not initialized".to_string()),
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn authorization<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.authorization = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for authorization failed".to_string()
            });
            self
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::ReportStartBody>,
            <V as std::convert::TryInto<types::ReportStartBody>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `ReportStartBody` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::ReportStartBody) -> types::builder::ReportStartBody,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/report/start`
        pub async fn send(self) -> Result<ResponseValue<types::ReportResult>, Error<()>> {
            let Self {
                client,
                authorization,
                body,
            } = self;
            let authorization = authorization.map_err(Error::InvalidRequest)?;
            let body = body
                .and_then(|v| types::ReportStartBody::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/report/start", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            header_map.append("Authorization", authorization.to_string().try_into()?);
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
                operation_id: "report_start",
            };
            client
                .__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
                .await
        }
    }
}

/// Items consumers will typically use such as the Client and
/// extension traits.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
}
