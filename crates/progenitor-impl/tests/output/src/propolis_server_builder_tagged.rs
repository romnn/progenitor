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

    ///`CrucibleOpts`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "lossy",
    ///    "read_only",
    ///    "target"
    ///  ],
    ///  "properties": {
    ///    "cert_pem": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "control": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "flush_timeout": {
    ///      "type": [
    ///        "integer",
    ///        "null"
    ///      ],
    ///      "format": "uint32",
    ///      "minimum": 0.0
    ///    },
    ///    "id": {
    ///      "type": "string",
    ///      "format": "uuid"
    ///    },
    ///    "key": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "key_pem": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "lossy": {
    ///      "type": "boolean"
    ///    },
    ///    "read_only": {
    ///      "type": "boolean"
    ///    },
    ///    "root_cert_pem": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "target": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct CrucibleOpts {
        pub cert_pem: ::std::option::Option<::std::string::String>,
        pub control: ::std::option::Option<::std::string::String>,
        pub flush_timeout: ::std::option::Option<u32>,
        pub id: ::uuid::Uuid,
        pub key: ::std::option::Option<::std::string::String>,
        pub key_pem: ::std::option::Option<::std::string::String>,
        pub lossy: bool,
        pub read_only: bool,
        pub root_cert_pem: ::std::option::Option<::std::string::String>,
        pub target: ::std::vec::Vec<::std::string::String>,
    }

    impl<'de> self::de::Build<'de> for CrucibleOpts {
        const NAME: &'static str = "CrucibleOpts";
        const FIELDS: &'static [&'static str] = &[
            "cert_pem",
            "control",
            "flush_timeout",
            "id",
            "key",
            "key_pem",
            "lossy",
            "read_only",
            "root_cert_pem",
            "target",
        ];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                cert_pem: self::de::defaulted(&mut fields, "cert_pem")?,
                control: self::de::defaulted(&mut fields, "control")?,
                flush_timeout: self::de::defaulted(&mut fields, "flush_timeout")?,
                id: self::de::required(&mut fields, "id")?,
                key: self::de::defaulted(&mut fields, "key")?,
                key_pem: self::de::defaulted(&mut fields, "key_pem")?,
                lossy: self::de::required(&mut fields, "lossy")?,
                read_only: self::de::required(&mut fields, "read_only")?,
                root_cert_pem: self::de::defaulted(&mut fields, "root_cert_pem")?,
                target: self::de::required(&mut fields, "target")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for CrucibleOpts {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for CrucibleOpts {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize
                + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.cert_pem))
                + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.control))
                + ::std::primitive::usize::from(!::std::option::Option::is_none(
                    &self.flush_timeout,
                ))
                + 1
                + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.key))
                + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.key_pem))
                + 1
                + 1
                + ::std::primitive::usize::from(!::std::option::Option::is_none(
                    &self.root_cert_pem,
                ))
                + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "CrucibleOpts", len)?;
            if !::std::option::Option::is_none(&self.cert_pem) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "cert_pem",
                    &self.cert_pem,
                )?;
            }
            if !::std::option::Option::is_none(&self.control) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "control",
                    &self.control,
                )?;
            }
            if !::std::option::Option::is_none(&self.flush_timeout) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "flush_timeout",
                    &self.flush_timeout,
                )?;
            }
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            if !::std::option::Option::is_none(&self.key) {
                ::serde::ser::SerializeStruct::serialize_field(&mut state, "key", &self.key)?;
            }
            if !::std::option::Option::is_none(&self.key_pem) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "key_pem",
                    &self.key_pem,
                )?;
            }
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "lossy", &self.lossy)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "read_only",
                &self.read_only,
            )?;
            if !::std::option::Option::is_none(&self.root_cert_pem) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "root_cert_pem",
                    &self.root_cert_pem,
                )?;
            }
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "target", &self.target)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl CrucibleOpts {
        pub fn builder() -> builder::CrucibleOpts {
            ::std::default::Default::default()
        }
    }

    ///`DiskAttachment`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "disk_id",
    ///    "generation_id",
    ///    "state"
    ///  ],
    ///  "properties": {
    ///    "disk_id": {
    ///      "type": "string",
    ///      "format": "uuid"
    ///    },
    ///    "generation_id": {
    ///      "type": "integer",
    ///      "format": "uint64",
    ///      "minimum": 0.0
    ///    },
    ///    "state": {
    ///      "$ref": "#/components/schemas/DiskAttachmentState"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct DiskAttachment {
        pub disk_id: ::uuid::Uuid,
        pub generation_id: u64,
        pub state: DiskAttachmentState,
    }

    impl<'de> self::de::Build<'de> for DiskAttachment {
        const NAME: &'static str = "DiskAttachment";
        const FIELDS: &'static [&'static str] = &["disk_id", "generation_id", "state"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                disk_id: self::de::required(&mut fields, "disk_id")?,
                generation_id: self::de::required(&mut fields, "generation_id")?,
                state: self::de::required(&mut fields, "state")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for DiskAttachment {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for DiskAttachment {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "DiskAttachment", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "disk_id", &self.disk_id)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "generation_id",
                &self.generation_id,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "state", &self.state)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl DiskAttachment {
        pub fn builder() -> builder::DiskAttachment {
            ::std::default::Default::default()
        }
    }

    ///`DiskAttachmentState`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "oneOf": [
    ///    {
    ///      "type": "string",
    ///      "enum": [
    ///        "Detached",
    ///        "Destroyed",
    ///        "Faulted"
    ///      ]
    ///    },
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "Attached"
    ///      ],
    ///      "properties": {
    ///        "Attached": {
    ///          "type": "string",
    ///          "format": "uuid"
    ///        }
    ///      },
    ///      "additionalProperties": false
    ///    }
    ///  ]
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub enum DiskAttachmentState {
        Detached,
        Destroyed,
        Faulted,
        Attached(::uuid::Uuid),
    }

    impl ::std::convert::From<::uuid::Uuid> for DiskAttachmentState {
        fn from(value: ::uuid::Uuid) -> Self {
            Self::Attached(value)
        }
    }

    ///`DiskRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "device",
    ///    "gen",
    ///    "name",
    ///    "read_only",
    ///    "slot",
    ///    "volume_construction_request"
    ///  ],
    ///  "properties": {
    ///    "device": {
    ///      "type": "string"
    ///    },
    ///    "gen": {
    ///      "type": "integer",
    ///      "format": "uint64",
    ///      "minimum": 0.0
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "read_only": {
    ///      "type": "boolean"
    ///    },
    ///    "slot": {
    ///      "$ref": "#/components/schemas/Slot"
    ///    },
    ///    "volume_construction_request": {
    ///      "$ref": "#/components/schemas/VolumeConstructionRequest"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct DiskRequest {
        pub device: ::std::string::String,
        pub gen_: u64,
        pub name: ::std::string::String,
        pub read_only: bool,
        pub slot: Slot,
        pub volume_construction_request: VolumeConstructionRequest,
    }

    impl<'de> self::de::Build<'de> for DiskRequest {
        const NAME: &'static str = "DiskRequest";
        const FIELDS: &'static [&'static str] = &[
            "device",
            "gen",
            "name",
            "read_only",
            "slot",
            "volume_construction_request",
        ];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                device: self::de::required(&mut fields, "device")?,
                gen_: self::de::required(&mut fields, "gen")?,
                name: self::de::required(&mut fields, "name")?,
                read_only: self::de::required(&mut fields, "read_only")?,
                slot: self::de::required(&mut fields, "slot")?,
                volume_construction_request: self::de::required(
                    &mut fields,
                    "volume_construction_request",
                )?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for DiskRequest {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for DiskRequest {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1 + 1 + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "DiskRequest", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "device", &self.device)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "gen", &self.gen_)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "read_only",
                &self.read_only,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "slot", &self.slot)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "volume_construction_request",
                &self.volume_construction_request,
            )?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl DiskRequest {
        pub fn builder() -> builder::DiskRequest {
            ::std::default::Default::default()
        }
    }

    ///Error information from a response.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "description": "Error information from a response.",
    ///  "type": "object",
    ///  "required": [
    ///    "message",
    ///    "request_id"
    ///  ],
    ///  "properties": {
    ///    "error_code": {
    ///      "type": "string"
    ///    },
    ///    "message": {
    ///      "type": "string"
    ///    },
    ///    "request_id": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct Error {
        pub error_code: ::std::option::Option<::std::string::String>,
        pub message: ::std::string::String,
        pub request_id: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for Error {
        const NAME: &'static str = "Error";
        const FIELDS: &'static [&'static str] = &["error_code", "message", "request_id"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                error_code: self::de::defaulted(&mut fields, "error_code")?,
                message: self::de::required(&mut fields, "message")?,
                request_id: self::de::required(&mut fields, "request_id")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for Error {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for Error {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize
                + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.error_code))
                + 1
                + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "Error", len)?;
            if !::std::option::Option::is_none(&self.error_code) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "error_code",
                    &self.error_code,
                )?;
            }
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "message", &self.message)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "request_id",
                &self.request_id,
            )?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl Error {
        pub fn builder() -> builder::Error {
            ::std::default::Default::default()
        }
    }

    ///`Instance`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "disks",
    ///    "nics",
    ///    "properties",
    ///    "state"
    ///  ],
    ///  "properties": {
    ///    "disks": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/DiskAttachment"
    ///      }
    ///    },
    ///    "nics": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NetworkInterface"
    ///      }
    ///    },
    ///    "properties": {
    ///      "$ref": "#/components/schemas/InstanceProperties"
    ///    },
    ///    "state": {
    ///      "$ref": "#/components/schemas/InstanceState"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct Instance {
        pub disks: ::std::vec::Vec<DiskAttachment>,
        pub nics: ::std::vec::Vec<NetworkInterface>,
        pub properties: InstanceProperties,
        pub state: InstanceState,
    }

    impl<'de> self::de::Build<'de> for Instance {
        const NAME: &'static str = "Instance";
        const FIELDS: &'static [&'static str] = &["disks", "nics", "properties", "state"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                disks: self::de::required(&mut fields, "disks")?,
                nics: self::de::required(&mut fields, "nics")?,
                properties: self::de::required(&mut fields, "properties")?,
                state: self::de::required(&mut fields, "state")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for Instance {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for Instance {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(serializer, "Instance", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "disks", &self.disks)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "nics", &self.nics)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "properties",
                &self.properties,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "state", &self.state)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl Instance {
        pub fn builder() -> builder::Instance {
            ::std::default::Default::default()
        }
    }

    ///`InstanceEnsureRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "properties"
    ///  ],
    ///  "properties": {
    ///    "cloud_init_bytes": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "disks": {
    ///      "default": [],
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/DiskRequest"
    ///      }
    ///    },
    ///    "migrate": {
    ///      "oneOf": [
    ///        {
    ///          "type": "null"
    ///        },
    ///        {
    ///          "allOf": [
    ///            {
    ///              "$ref": "#/components/schemas/InstanceMigrateInitiateRequest"
    ///            }
    ///          ]
    ///        }
    ///      ]
    ///    },
    ///    "nics": {
    ///      "default": [],
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NetworkInterfaceRequest"
    ///      }
    ///    },
    ///    "properties": {
    ///      "$ref": "#/components/schemas/InstanceProperties"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct InstanceEnsureRequest {
        pub cloud_init_bytes: ::std::option::Option<::std::string::String>,
        pub disks: ::std::vec::Vec<DiskRequest>,
        pub migrate: ::std::option::Option<InstanceMigrateInitiateRequest>,
        pub nics: ::std::vec::Vec<NetworkInterfaceRequest>,
        pub properties: InstanceProperties,
    }

    impl<'de> self::de::Build<'de> for InstanceEnsureRequest {
        const NAME: &'static str = "InstanceEnsureRequest";
        const FIELDS: &'static [&'static str] =
            &["cloud_init_bytes", "disks", "migrate", "nics", "properties"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                cloud_init_bytes: self::de::defaulted(&mut fields, "cloud_init_bytes")?,
                disks: self::de::defaulted(&mut fields, "disks")?,
                migrate: self::de::defaulted(&mut fields, "migrate")?,
                nics: self::de::defaulted(&mut fields, "nics")?,
                properties: self::de::required(&mut fields, "properties")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceEnsureRequest {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for InstanceEnsureRequest {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize
                + ::std::primitive::usize::from(!::std::option::Option::is_none(
                    &self.cloud_init_bytes,
                ))
                + ::std::primitive::usize::from(!::std::vec::Vec::is_empty(&self.disks))
                + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.migrate))
                + ::std::primitive::usize::from(!::std::vec::Vec::is_empty(&self.nics))
                + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "InstanceEnsureRequest", len)?;
            if !::std::option::Option::is_none(&self.cloud_init_bytes) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "cloud_init_bytes",
                    &self.cloud_init_bytes,
                )?;
            }
            if !::std::vec::Vec::is_empty(&self.disks) {
                ::serde::ser::SerializeStruct::serialize_field(&mut state, "disks", &self.disks)?;
            }
            if !::std::option::Option::is_none(&self.migrate) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "migrate",
                    &self.migrate,
                )?;
            }
            if !::std::vec::Vec::is_empty(&self.nics) {
                ::serde::ser::SerializeStruct::serialize_field(&mut state, "nics", &self.nics)?;
            }
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "properties",
                &self.properties,
            )?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl InstanceEnsureRequest {
        pub fn builder() -> builder::InstanceEnsureRequest {
            ::std::default::Default::default()
        }
    }

    ///`InstanceEnsureResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "properties": {
    ///    "migrate": {
    ///      "oneOf": [
    ///        {
    ///          "type": "null"
    ///        },
    ///        {
    ///          "allOf": [
    ///            {
    ///              "$ref": "#/components/schemas/InstanceMigrateInitiateResponse"
    ///            }
    ///          ]
    ///        }
    ///      ]
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct InstanceEnsureResponse {
        pub migrate: ::std::option::Option<InstanceMigrateInitiateResponse>,
    }

    impl<'de> self::de::Build<'de> for InstanceEnsureResponse {
        const NAME: &'static str = "InstanceEnsureResponse";
        const FIELDS: &'static [&'static str] = &["migrate"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                migrate: self::de::defaulted(&mut fields, "migrate")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceEnsureResponse {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for InstanceEnsureResponse {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize
                + ::std::primitive::usize::from(!::std::option::Option::is_none(&self.migrate));
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "InstanceEnsureResponse", len)?;
            if !::std::option::Option::is_none(&self.migrate) {
                ::serde::ser::SerializeStruct::serialize_field(
                    &mut state,
                    "migrate",
                    &self.migrate,
                )?;
            }
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl ::std::default::Default for InstanceEnsureResponse {
        fn default() -> Self {
            Self {
                migrate: ::std::default::Default::default(),
            }
        }
    }

    impl InstanceEnsureResponse {
        pub fn builder() -> builder::InstanceEnsureResponse {
            ::std::default::Default::default()
        }
    }

    ///`InstanceGetResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "instance"
    ///  ],
    ///  "properties": {
    ///    "instance": {
    ///      "$ref": "#/components/schemas/Instance"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct InstanceGetResponse {
        pub instance: Instance,
    }

    impl<'de> self::de::Build<'de> for InstanceGetResponse {
        const NAME: &'static str = "InstanceGetResponse";
        const FIELDS: &'static [&'static str] = &["instance"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                instance: self::de::required(&mut fields, "instance")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceGetResponse {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for InstanceGetResponse {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "InstanceGetResponse", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "instance", &self.instance)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl InstanceGetResponse {
        pub fn builder() -> builder::InstanceGetResponse {
            ::std::default::Default::default()
        }
    }

    ///`InstanceMigrateInitiateRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "migration_id",
    ///    "src_addr",
    ///    "src_uuid"
    ///  ],
    ///  "properties": {
    ///    "migration_id": {
    ///      "type": "string",
    ///      "format": "uuid"
    ///    },
    ///    "src_addr": {
    ///      "type": "string"
    ///    },
    ///    "src_uuid": {
    ///      "type": "string",
    ///      "format": "uuid"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct InstanceMigrateInitiateRequest {
        pub migration_id: ::uuid::Uuid,
        pub src_addr: ::std::string::String,
        pub src_uuid: ::uuid::Uuid,
    }

    impl<'de> self::de::Build<'de> for InstanceMigrateInitiateRequest {
        const NAME: &'static str = "InstanceMigrateInitiateRequest";
        const FIELDS: &'static [&'static str] = &["migration_id", "src_addr", "src_uuid"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                migration_id: self::de::required(&mut fields, "migration_id")?,
                src_addr: self::de::required(&mut fields, "src_addr")?,
                src_uuid: self::de::required(&mut fields, "src_uuid")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceMigrateInitiateRequest {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for InstanceMigrateInitiateRequest {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(
                serializer,
                "InstanceMigrateInitiateRequest",
                len,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "migration_id",
                &self.migration_id,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "src_addr", &self.src_addr)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "src_uuid", &self.src_uuid)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl InstanceMigrateInitiateRequest {
        pub fn builder() -> builder::InstanceMigrateInitiateRequest {
            ::std::default::Default::default()
        }
    }

    ///`InstanceMigrateInitiateResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "migration_id"
    ///  ],
    ///  "properties": {
    ///    "migration_id": {
    ///      "type": "string",
    ///      "format": "uuid"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct InstanceMigrateInitiateResponse {
        pub migration_id: ::uuid::Uuid,
    }

    impl<'de> self::de::Build<'de> for InstanceMigrateInitiateResponse {
        const NAME: &'static str = "InstanceMigrateInitiateResponse";
        const FIELDS: &'static [&'static str] = &["migration_id"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                migration_id: self::de::required(&mut fields, "migration_id")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceMigrateInitiateResponse {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for InstanceMigrateInitiateResponse {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1;
            let mut state = ::serde::Serializer::serialize_struct(
                serializer,
                "InstanceMigrateInitiateResponse",
                len,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "migration_id",
                &self.migration_id,
            )?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl InstanceMigrateInitiateResponse {
        pub fn builder() -> builder::InstanceMigrateInitiateResponse {
            ::std::default::Default::default()
        }
    }

    ///`InstanceMigrateStatusRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "migration_id"
    ///  ],
    ///  "properties": {
    ///    "migration_id": {
    ///      "type": "string",
    ///      "format": "uuid"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    pub use self::InstanceMigrateInitiateResponse as InstanceMigrateStatusRequest;
    ///`InstanceMigrateStatusResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "state"
    ///  ],
    ///  "properties": {
    ///    "state": {
    ///      "$ref": "#/components/schemas/MigrationState"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct InstanceMigrateStatusResponse {
        pub state: MigrationState,
    }

    impl<'de> self::de::Build<'de> for InstanceMigrateStatusResponse {
        const NAME: &'static str = "InstanceMigrateStatusResponse";
        const FIELDS: &'static [&'static str] = &["state"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                state: self::de::required(&mut fields, "state")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceMigrateStatusResponse {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for InstanceMigrateStatusResponse {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1;
            let mut state = ::serde::Serializer::serialize_struct(
                serializer,
                "InstanceMigrateStatusResponse",
                len,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "state", &self.state)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl InstanceMigrateStatusResponse {
        pub fn builder() -> builder::InstanceMigrateStatusResponse {
            ::std::default::Default::default()
        }
    }

    ///`InstanceProperties`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "bootrom_id",
    ///    "description",
    ///    "id",
    ///    "image_id",
    ///    "memory",
    ///    "name",
    ///    "vcpus"
    ///  ],
    ///  "properties": {
    ///    "bootrom_id": {
    ///      "description": "ID of the bootrom used to initialize this Instance.",
    ///      "type": "string",
    ///      "format": "uuid"
    ///    },
    ///    "description": {
    ///      "description": "Free-form text description of an Instance.",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "description": "Unique identifier for this Instance.",
    ///      "type": "string",
    ///      "format": "uuid"
    ///    },
    ///    "image_id": {
    ///      "description": "ID of the image used to initialize this Instance.",
    ///      "type": "string",
    ///      "format": "uuid"
    ///    },
    ///    "memory": {
    ///      "description": "Size of memory allocated to the Instance, in MiB.",
    ///      "type": "integer",
    ///      "format": "uint64",
    ///      "minimum": 0.0
    ///    },
    ///    "name": {
    ///      "description": "Human-readable name of the Instance.",
    ///      "type": "string"
    ///    },
    ///    "vcpus": {
    ///      "description": "Number of vCPUs to be allocated to the Instance.",
    ///      "type": "integer",
    ///      "format": "uint8",
    ///      "minimum": 0.0
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct InstanceProperties {
        ///ID of the bootrom used to initialize this Instance.
        pub bootrom_id: ::uuid::Uuid,
        ///Free-form text description of an Instance.
        pub description: ::std::string::String,
        ///Unique identifier for this Instance.
        pub id: ::uuid::Uuid,
        ///ID of the image used to initialize this Instance.
        pub image_id: ::uuid::Uuid,
        ///Size of memory allocated to the Instance, in MiB.
        pub memory: u64,
        ///Human-readable name of the Instance.
        pub name: ::std::string::String,
        ///Number of vCPUs to be allocated to the Instance.
        pub vcpus: u8,
    }

    impl<'de> self::de::Build<'de> for InstanceProperties {
        const NAME: &'static str = "InstanceProperties";
        const FIELDS: &'static [&'static str] = &[
            "bootrom_id",
            "description",
            "id",
            "image_id",
            "memory",
            "name",
            "vcpus",
        ];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                bootrom_id: self::de::required(&mut fields, "bootrom_id")?,
                description: self::de::required(&mut fields, "description")?,
                id: self::de::required(&mut fields, "id")?,
                image_id: self::de::required(&mut fields, "image_id")?,
                memory: self::de::required(&mut fields, "memory")?,
                name: self::de::required(&mut fields, "name")?,
                vcpus: self::de::required(&mut fields, "vcpus")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceProperties {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for InstanceProperties {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1 + 1 + 1 + 1 + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "InstanceProperties", len)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "bootrom_id",
                &self.bootrom_id,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "description",
                &self.description,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "id", &self.id)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "image_id", &self.image_id)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "memory", &self.memory)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "vcpus", &self.vcpus)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl InstanceProperties {
        pub fn builder() -> builder::InstanceProperties {
            ::std::default::Default::default()
        }
    }

    ///Current state of an Instance.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "description": "Current state of an Instance.",
    ///  "type": "string",
    ///  "enum": [
    ///    "Creating",
    ///    "Starting",
    ///    "Running",
    ///    "Stopping",
    ///    "Stopped",
    ///    "Rebooting",
    ///    "Migrating",
    ///    "Repairing",
    ///    "Failed",
    ///    "Destroyed"
    ///  ]
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum InstanceState {
        Creating,
        Starting,
        Running,
        Stopping,
        Stopped,
        Rebooting,
        Migrating,
        Repairing,
        Failed,
        Destroyed,
    }

    impl self::de::UnitEnum for InstanceState {
        const NAME: &'static str = "InstanceState";
        const VARIANTS: &'static [&'static str] = &[
            "Creating",
            "Starting",
            "Running",
            "Stopping",
            "Stopped",
            "Rebooting",
            "Migrating",
            "Repairing",
            "Failed",
            "Destroyed",
        ];
        fn from_index(index: usize) -> Self {
            match index {
                0usize => Self::Creating,
                1usize => Self::Starting,
                2usize => Self::Running,
                3usize => Self::Stopping,
                4usize => Self::Stopped,
                5usize => Self::Rebooting,
                6usize => Self::Migrating,
                7usize => Self::Repairing,
                8usize => Self::Failed,
                9usize => Self::Destroyed,
                _ => unreachable!("variant index out of range"),
            }
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceState {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_unit_enum(deserializer)
        }
    }

    impl ::std::fmt::Display for InstanceState {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Creating => f.write_str("Creating"),
                Self::Starting => f.write_str("Starting"),
                Self::Running => f.write_str("Running"),
                Self::Stopping => f.write_str("Stopping"),
                Self::Stopped => f.write_str("Stopped"),
                Self::Rebooting => f.write_str("Rebooting"),
                Self::Migrating => f.write_str("Migrating"),
                Self::Repairing => f.write_str("Repairing"),
                Self::Failed => f.write_str("Failed"),
                Self::Destroyed => f.write_str("Destroyed"),
            }
        }
    }

    impl ::std::str::FromStr for InstanceState {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "Creating" => Ok(Self::Creating),
                "Starting" => Ok(Self::Starting),
                "Running" => Ok(Self::Running),
                "Stopping" => Ok(Self::Stopping),
                "Stopped" => Ok(Self::Stopped),
                "Rebooting" => Ok(Self::Rebooting),
                "Migrating" => Ok(Self::Migrating),
                "Repairing" => Ok(Self::Repairing),
                "Failed" => Ok(Self::Failed),
                "Destroyed" => Ok(Self::Destroyed),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for InstanceState {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for InstanceState {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for InstanceState {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`InstanceStateMonitorRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "gen"
    ///  ],
    ///  "properties": {
    ///    "gen": {
    ///      "type": "integer",
    ///      "format": "uint64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct InstanceStateMonitorRequest {
        pub gen_: u64,
    }

    impl<'de> self::de::Build<'de> for InstanceStateMonitorRequest {
        const NAME: &'static str = "InstanceStateMonitorRequest";
        const FIELDS: &'static [&'static str] = &["gen"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                gen_: self::de::required(&mut fields, "gen")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceStateMonitorRequest {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for InstanceStateMonitorRequest {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1;
            let mut state = ::serde::Serializer::serialize_struct(
                serializer,
                "InstanceStateMonitorRequest",
                len,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "gen", &self.gen_)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl InstanceStateMonitorRequest {
        pub fn builder() -> builder::InstanceStateMonitorRequest {
            ::std::default::Default::default()
        }
    }

    ///`InstanceStateMonitorResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "gen",
    ///    "state"
    ///  ],
    ///  "properties": {
    ///    "gen": {
    ///      "type": "integer",
    ///      "format": "uint64",
    ///      "minimum": 0.0
    ///    },
    ///    "state": {
    ///      "$ref": "#/components/schemas/InstanceState"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct InstanceStateMonitorResponse {
        pub gen_: u64,
        pub state: InstanceState,
    }

    impl<'de> self::de::Build<'de> for InstanceStateMonitorResponse {
        const NAME: &'static str = "InstanceStateMonitorResponse";
        const FIELDS: &'static [&'static str] = &["gen", "state"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                gen_: self::de::required(&mut fields, "gen")?,
                state: self::de::required(&mut fields, "state")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceStateMonitorResponse {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for InstanceStateMonitorResponse {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1;
            let mut state = ::serde::Serializer::serialize_struct(
                serializer,
                "InstanceStateMonitorResponse",
                len,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "gen", &self.gen_)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "state", &self.state)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl InstanceStateMonitorResponse {
        pub fn builder() -> builder::InstanceStateMonitorResponse {
            ::std::default::Default::default()
        }
    }

    ///`InstanceStateRequested`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "string",
    ///  "enum": [
    ///    "Run",
    ///    "Stop",
    ///    "Reboot",
    ///    "MigrateStart"
    ///  ]
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum InstanceStateRequested {
        Run,
        Stop,
        Reboot,
        MigrateStart,
    }

    impl self::de::UnitEnum for InstanceStateRequested {
        const NAME: &'static str = "InstanceStateRequested";
        const VARIANTS: &'static [&'static str] = &["Run", "Stop", "Reboot", "MigrateStart"];
        fn from_index(index: usize) -> Self {
            match index {
                0usize => Self::Run,
                1usize => Self::Stop,
                2usize => Self::Reboot,
                3usize => Self::MigrateStart,
                _ => unreachable!("variant index out of range"),
            }
        }
    }

    impl<'de> ::serde::Deserialize<'de> for InstanceStateRequested {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_unit_enum(deserializer)
        }
    }

    impl ::std::fmt::Display for InstanceStateRequested {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Run => f.write_str("Run"),
                Self::Stop => f.write_str("Stop"),
                Self::Reboot => f.write_str("Reboot"),
                Self::MigrateStart => f.write_str("MigrateStart"),
            }
        }
    }

    impl ::std::str::FromStr for InstanceStateRequested {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "Run" => Ok(Self::Run),
                "Stop" => Ok(Self::Stop),
                "Reboot" => Ok(Self::Reboot),
                "MigrateStart" => Ok(Self::MigrateStart),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for InstanceStateRequested {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for InstanceStateRequested {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for InstanceStateRequested {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`MigrationState`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "string",
    ///  "enum": [
    ///    "Sync",
    ///    "RamPush",
    ///    "Pause",
    ///    "RamPushDirty",
    ///    "Device",
    ///    "Arch",
    ///    "Resume",
    ///    "RamPull",
    ///    "Finish",
    ///    "Error"
    ///  ]
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum MigrationState {
        Sync,
        RamPush,
        Pause,
        RamPushDirty,
        Device,
        Arch,
        Resume,
        RamPull,
        Finish,
        Error,
    }

    impl self::de::UnitEnum for MigrationState {
        const NAME: &'static str = "MigrationState";
        const VARIANTS: &'static [&'static str] = &[
            "Sync",
            "RamPush",
            "Pause",
            "RamPushDirty",
            "Device",
            "Arch",
            "Resume",
            "RamPull",
            "Finish",
            "Error",
        ];
        fn from_index(index: usize) -> Self {
            match index {
                0usize => Self::Sync,
                1usize => Self::RamPush,
                2usize => Self::Pause,
                3usize => Self::RamPushDirty,
                4usize => Self::Device,
                5usize => Self::Arch,
                6usize => Self::Resume,
                7usize => Self::RamPull,
                8usize => Self::Finish,
                9usize => Self::Error,
                _ => unreachable!("variant index out of range"),
            }
        }
    }

    impl<'de> ::serde::Deserialize<'de> for MigrationState {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_unit_enum(deserializer)
        }
    }

    impl ::std::fmt::Display for MigrationState {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Sync => f.write_str("Sync"),
                Self::RamPush => f.write_str("RamPush"),
                Self::Pause => f.write_str("Pause"),
                Self::RamPushDirty => f.write_str("RamPushDirty"),
                Self::Device => f.write_str("Device"),
                Self::Arch => f.write_str("Arch"),
                Self::Resume => f.write_str("Resume"),
                Self::RamPull => f.write_str("RamPull"),
                Self::Finish => f.write_str("Finish"),
                Self::Error => f.write_str("Error"),
            }
        }
    }

    impl ::std::str::FromStr for MigrationState {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "Sync" => Ok(Self::Sync),
                "RamPush" => Ok(Self::RamPush),
                "Pause" => Ok(Self::Pause),
                "RamPushDirty" => Ok(Self::RamPushDirty),
                "Device" => Ok(Self::Device),
                "Arch" => Ok(Self::Arch),
                "Resume" => Ok(Self::Resume),
                "RamPull" => Ok(Self::RamPull),
                "Finish" => Ok(Self::Finish),
                "Error" => Ok(Self::Error),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for MigrationState {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for MigrationState {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for MigrationState {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`NetworkInterface`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "attachment",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "attachment": {
    ///      "$ref": "#/components/schemas/NetworkInterfaceAttachmentState"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct NetworkInterface {
        pub attachment: NetworkInterfaceAttachmentState,
        pub name: ::std::string::String,
    }

    impl<'de> self::de::Build<'de> for NetworkInterface {
        const NAME: &'static str = "NetworkInterface";
        const FIELDS: &'static [&'static str] = &["attachment", "name"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                attachment: self::de::required(&mut fields, "attachment")?,
                name: self::de::required(&mut fields, "name")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for NetworkInterface {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for NetworkInterface {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "NetworkInterface", len)?;
            ::serde::ser::SerializeStruct::serialize_field(
                &mut state,
                "attachment",
                &self.attachment,
            )?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl NetworkInterface {
        pub fn builder() -> builder::NetworkInterface {
            ::std::default::Default::default()
        }
    }

    ///`NetworkInterfaceAttachmentState`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "oneOf": [
    ///    {
    ///      "type": "string",
    ///      "enum": [
    ///        "Detached",
    ///        "Faulted"
    ///      ]
    ///    },
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "Attached"
    ///      ],
    ///      "properties": {
    ///        "Attached": {
    ///          "$ref": "#/components/schemas/Slot"
    ///        }
    ///      },
    ///      "additionalProperties": false
    ///    }
    ///  ]
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub enum NetworkInterfaceAttachmentState {
        Detached,
        Faulted,
        Attached(Slot),
    }

    impl ::std::convert::From<Slot> for NetworkInterfaceAttachmentState {
        fn from(value: Slot) -> Self {
            Self::Attached(value)
        }
    }

    ///`NetworkInterfaceRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "name",
    ///    "slot"
    ///  ],
    ///  "properties": {
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "slot": {
    ///      "$ref": "#/components/schemas/Slot"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(Clone, Debug)]
    pub struct NetworkInterfaceRequest {
        pub name: ::std::string::String,
        pub slot: Slot,
    }

    impl<'de> self::de::Build<'de> for NetworkInterfaceRequest {
        const NAME: &'static str = "NetworkInterfaceRequest";
        const FIELDS: &'static [&'static str] = &["name", "slot"];
        fn build<E>(mut fields: self::de::Fields<'de>) -> ::std::result::Result<Self, E>
        where
            E: ::serde::de::Error,
        {
            let value = Self {
                name: self::de::required(&mut fields, "name")?,
                slot: self::de::required(&mut fields, "slot")?,
            };
            ::std::result::Result::Ok(value)
        }
    }

    impl<'de> ::serde::Deserialize<'de> for NetworkInterfaceRequest {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            self::de::deserialize_struct(deserializer)
        }
    }

    impl ::serde::Serialize for NetworkInterfaceRequest {
        fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let len = 0usize + 1 + 1;
            let mut state =
                ::serde::Serializer::serialize_struct(serializer, "NetworkInterfaceRequest", len)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "name", &self.name)?;
            ::serde::ser::SerializeStruct::serialize_field(&mut state, "slot", &self.slot)?;
            ::serde::ser::SerializeStruct::end(state)
        }
    }

    impl NetworkInterfaceRequest {
        pub fn builder() -> builder::NetworkInterfaceRequest {
            ::std::default::Default::default()
        }
    }

    ///A stable index which is translated by Propolis into a PCI BDF, visible
    /// to the guest.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "description": "A stable index which is translated by Propolis into a PCI BDF, visible to the guest.",
    ///  "type": "integer",
    ///  "format": "uint8",
    ///  "minimum": 0.0
    /// }
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    #[serde(transparent)]
    pub struct Slot(pub u8);
    impl ::std::ops::Deref for Slot {
        type Target = u8;
        fn deref(&self) -> &u8 {
            &self.0
        }
    }

    impl ::std::convert::From<Slot> for u8 {
        fn from(value: Slot) -> Self {
            value.0
        }
    }

    impl ::std::convert::From<u8> for Slot {
        fn from(value: u8) -> Self {
            Self(value)
        }
    }

    impl ::std::str::FromStr for Slot {
        type Err = <u8 as ::std::str::FromStr>::Err;
        fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
            Ok(Self(value.parse()?))
        }
    }

    impl ::std::convert::TryFrom<&str> for Slot {
        type Error = <u8 as ::std::str::FromStr>::Err;
        fn try_from(value: &str) -> ::std::result::Result<Self, Self::Error> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<String> for Slot {
        type Error = <u8 as ::std::str::FromStr>::Err;
        fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
            value.parse()
        }
    }

    impl ::std::fmt::Display for Slot {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }

    ///`VolumeConstructionRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "oneOf": [
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "block_size",
    ///        "id",
    ///        "sub_volumes",
    ///        "type"
    ///      ],
    ///      "properties": {
    ///        "block_size": {
    ///          "type": "integer",
    ///          "format": "uint64",
    ///          "minimum": 0.0
    ///        },
    ///        "id": {
    ///          "type": "string",
    ///          "format": "uuid"
    ///        },
    ///        "read_only_parent": {
    ///          "oneOf": [
    ///            {
    ///              "type": "null"
    ///            },
    ///            {
    ///              "allOf": [
    ///                {
    ///                  "$ref": "#/components/schemas/VolumeConstructionRequest"
    ///                }
    ///              ]
    ///            }
    ///          ]
    ///        },
    ///        "sub_volumes": {
    ///          "type": "array",
    ///          "items": {
    ///            "$ref": "#/components/schemas/VolumeConstructionRequest"
    ///          }
    ///        },
    ///        "type": {
    ///          "type": "string",
    ///          "enum": [
    ///            "volume"
    ///          ]
    ///        }
    ///      }
    ///    },
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "block_size",
    ///        "id",
    ///        "type",
    ///        "url"
    ///      ],
    ///      "properties": {
    ///        "block_size": {
    ///          "type": "integer",
    ///          "format": "uint64",
    ///          "minimum": 0.0
    ///        },
    ///        "id": {
    ///          "type": "string",
    ///          "format": "uuid"
    ///        },
    ///        "type": {
    ///          "type": "string",
    ///          "enum": [
    ///            "url"
    ///          ]
    ///        },
    ///        "url": {
    ///          "type": "string"
    ///        }
    ///      }
    ///    },
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "block_size",
    ///        "gen",
    ///        "opts",
    ///        "type"
    ///      ],
    ///      "properties": {
    ///        "block_size": {
    ///          "type": "integer",
    ///          "format": "uint64",
    ///          "minimum": 0.0
    ///        },
    ///        "gen": {
    ///          "type": "integer",
    ///          "format": "uint64",
    ///          "minimum": 0.0
    ///        },
    ///        "opts": {
    ///          "$ref": "#/components/schemas/CrucibleOpts"
    ///        },
    ///        "type": {
    ///          "type": "string",
    ///          "enum": [
    ///            "region"
    ///          ]
    ///        }
    ///      }
    ///    },
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "block_size",
    ///        "id",
    ///        "path",
    ///        "type"
    ///      ],
    ///      "properties": {
    ///        "block_size": {
    ///          "type": "integer",
    ///          "format": "uint64",
    ///          "minimum": 0.0
    ///        },
    ///        "id": {
    ///          "type": "string",
    ///          "format": "uuid"
    ///        },
    ///        "path": {
    ///          "type": "string"
    ///        },
    ///        "type": {
    ///          "type": "string",
    ///          "enum": [
    ///            "file"
    ///          ]
    ///        }
    ///      }
    ///    }
    ///  ]
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum VolumeConstructionRequest {
        #[serde(rename = "volume")]
        Volume {
            block_size: u64,
            id: ::uuid::Uuid,
            #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
            read_only_parent: ::std::option::Option<::std::boxed::Box<VolumeConstructionRequest>>,
            sub_volumes: ::std::vec::Vec<VolumeConstructionRequest>,
        },
        #[serde(rename = "url")]
        Url {
            block_size: u64,
            id: ::uuid::Uuid,
            url: ::std::string::String,
        },
        #[serde(rename = "region")]
        Region {
            block_size: u64,
            #[serde(rename = "gen")]
            gen_: u64,
            opts: CrucibleOpts,
        },
        #[serde(rename = "file")]
        File {
            block_size: u64,
            id: ::uuid::Uuid,
            path: ::std::string::String,
        },
    }

    /// Types for composing complex structures.
    pub mod builder {
        #[derive(Clone, Debug)]
        pub struct CrucibleOpts {
            cert_pem: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            control: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            flush_timeout: ::std::result::Result<::std::option::Option<u32>, ::std::string::String>,
            id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
            key: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            key_pem: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            lossy: ::std::result::Result<bool, ::std::string::String>,
            read_only: ::std::result::Result<bool, ::std::string::String>,
            root_cert_pem: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            target: ::std::result::Result<
                ::std::vec::Vec<::std::string::String>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for CrucibleOpts {
            fn default() -> Self {
                Self {
                    cert_pem: Ok(::std::default::Default::default()),
                    control: Ok(::std::default::Default::default()),
                    flush_timeout: Ok(::std::default::Default::default()),
                    id: Err("no value supplied for id".to_string()),
                    key: Ok(::std::default::Default::default()),
                    key_pem: Ok(::std::default::Default::default()),
                    lossy: Err("no value supplied for lossy".to_string()),
                    read_only: Err("no value supplied for read_only".to_string()),
                    root_cert_pem: Ok(::std::default::Default::default()),
                    target: Err("no value supplied for target".to_string()),
                }
            }
        }

        impl CrucibleOpts {
            pub fn cert_pem<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.cert_pem = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for cert_pem: {e}"));
                self
            }
            pub fn control<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.control = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for control: {e}"));
                self
            }
            pub fn flush_timeout<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<u32>>,
                T::Error: ::std::fmt::Display,
            {
                self.flush_timeout = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for flush_timeout: {e}"));
                self
            }
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::uuid::Uuid>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
            pub fn key<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.key = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for key: {e}"));
                self
            }
            pub fn key_pem<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.key_pem = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for key_pem: {e}"));
                self
            }
            pub fn lossy<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.lossy = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for lossy: {e}"));
                self
            }
            pub fn read_only<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.read_only = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for read_only: {e}"));
                self
            }
            pub fn root_cert_pem<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.root_cert_pem = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for root_cert_pem: {e}"));
                self
            }
            pub fn target<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.target = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for target: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<CrucibleOpts> for super::CrucibleOpts {
            type Error = super::error::ConversionError;
            fn try_from(
                value: CrucibleOpts,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    cert_pem: value.cert_pem?,
                    control: value.control?,
                    flush_timeout: value.flush_timeout?,
                    id: value.id?,
                    key: value.key?,
                    key_pem: value.key_pem?,
                    lossy: value.lossy?,
                    read_only: value.read_only?,
                    root_cert_pem: value.root_cert_pem?,
                    target: value.target?,
                })
            }
        }

        impl ::std::convert::From<super::CrucibleOpts> for CrucibleOpts {
            fn from(value: super::CrucibleOpts) -> Self {
                Self {
                    cert_pem: Ok(value.cert_pem),
                    control: Ok(value.control),
                    flush_timeout: Ok(value.flush_timeout),
                    id: Ok(value.id),
                    key: Ok(value.key),
                    key_pem: Ok(value.key_pem),
                    lossy: Ok(value.lossy),
                    read_only: Ok(value.read_only),
                    root_cert_pem: Ok(value.root_cert_pem),
                    target: Ok(value.target),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct DiskAttachment {
            disk_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
            generation_id: ::std::result::Result<u64, ::std::string::String>,
            state: ::std::result::Result<super::DiskAttachmentState, ::std::string::String>,
        }

        impl ::std::default::Default for DiskAttachment {
            fn default() -> Self {
                Self {
                    disk_id: Err("no value supplied for disk_id".to_string()),
                    generation_id: Err("no value supplied for generation_id".to_string()),
                    state: Err("no value supplied for state".to_string()),
                }
            }
        }

        impl DiskAttachment {
            pub fn disk_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::uuid::Uuid>,
                T::Error: ::std::fmt::Display,
            {
                self.disk_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for disk_id: {e}"));
                self
            }
            pub fn generation_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u64>,
                T::Error: ::std::fmt::Display,
            {
                self.generation_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for generation_id: {e}"));
                self
            }
            pub fn state<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::DiskAttachmentState>,
                T::Error: ::std::fmt::Display,
            {
                self.state = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for state: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<DiskAttachment> for super::DiskAttachment {
            type Error = super::error::ConversionError;
            fn try_from(
                value: DiskAttachment,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    disk_id: value.disk_id?,
                    generation_id: value.generation_id?,
                    state: value.state?,
                })
            }
        }

        impl ::std::convert::From<super::DiskAttachment> for DiskAttachment {
            fn from(value: super::DiskAttachment) -> Self {
                Self {
                    disk_id: Ok(value.disk_id),
                    generation_id: Ok(value.generation_id),
                    state: Ok(value.state),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct DiskRequest {
            device: ::std::result::Result<::std::string::String, ::std::string::String>,
            gen_: ::std::result::Result<u64, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            read_only: ::std::result::Result<bool, ::std::string::String>,
            slot: ::std::result::Result<super::Slot, ::std::string::String>,
            volume_construction_request:
                ::std::result::Result<super::VolumeConstructionRequest, ::std::string::String>,
        }

        impl ::std::default::Default for DiskRequest {
            fn default() -> Self {
                Self {
                    device: Err("no value supplied for device".to_string()),
                    gen_: Err("no value supplied for gen_".to_string()),
                    name: Err("no value supplied for name".to_string()),
                    read_only: Err("no value supplied for read_only".to_string()),
                    slot: Err("no value supplied for slot".to_string()),
                    volume_construction_request: Err("no value supplied for \
                                                      volume_construction_request"
                        .to_string()),
                }
            }
        }

        impl DiskRequest {
            pub fn device<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.device = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for device: {e}"));
                self
            }
            pub fn gen_<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u64>,
                T::Error: ::std::fmt::Display,
            {
                self.gen_ = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for gen_: {e}"));
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
            pub fn read_only<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.read_only = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for read_only: {e}"));
                self
            }
            pub fn slot<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Slot>,
                T::Error: ::std::fmt::Display,
            {
                self.slot = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for slot: {e}"));
                self
            }
            pub fn volume_construction_request<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::VolumeConstructionRequest>,
                T::Error: ::std::fmt::Display,
            {
                self.volume_construction_request = value.try_into().map_err(|e| {
                    format!("error converting supplied value for volume_construction_request: {e}")
                });
                self
            }
        }

        impl ::std::convert::TryFrom<DiskRequest> for super::DiskRequest {
            type Error = super::error::ConversionError;
            fn try_from(
                value: DiskRequest,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    device: value.device?,
                    gen_: value.gen_?,
                    name: value.name?,
                    read_only: value.read_only?,
                    slot: value.slot?,
                    volume_construction_request: value.volume_construction_request?,
                })
            }
        }

        impl ::std::convert::From<super::DiskRequest> for DiskRequest {
            fn from(value: super::DiskRequest) -> Self {
                Self {
                    device: Ok(value.device),
                    gen_: Ok(value.gen_),
                    name: Ok(value.name),
                    read_only: Ok(value.read_only),
                    slot: Ok(value.slot),
                    volume_construction_request: Ok(value.volume_construction_request),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Error {
            error_code: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            message: ::std::result::Result<::std::string::String, ::std::string::String>,
            request_id: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for Error {
            fn default() -> Self {
                Self {
                    error_code: Ok(::std::default::Default::default()),
                    message: Err("no value supplied for message".to_string()),
                    request_id: Err("no value supplied for request_id".to_string()),
                }
            }
        }

        impl Error {
            pub fn error_code<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.error_code = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for error_code: {e}"));
                self
            }
            pub fn message<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.message = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for message: {e}"));
                self
            }
            pub fn request_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.request_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for request_id: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<Error> for super::Error {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Error,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    error_code: value.error_code?,
                    message: value.message?,
                    request_id: value.request_id?,
                })
            }
        }

        impl ::std::convert::From<super::Error> for Error {
            fn from(value: super::Error) -> Self {
                Self {
                    error_code: Ok(value.error_code),
                    message: Ok(value.message),
                    request_id: Ok(value.request_id),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Instance {
            disks: ::std::result::Result<
                ::std::vec::Vec<super::DiskAttachment>,
                ::std::string::String,
            >,
            nics: ::std::result::Result<
                ::std::vec::Vec<super::NetworkInterface>,
                ::std::string::String,
            >,
            properties: ::std::result::Result<super::InstanceProperties, ::std::string::String>,
            state: ::std::result::Result<super::InstanceState, ::std::string::String>,
        }

        impl ::std::default::Default for Instance {
            fn default() -> Self {
                Self {
                    disks: Err("no value supplied for disks".to_string()),
                    nics: Err("no value supplied for nics".to_string()),
                    properties: Err("no value supplied for properties".to_string()),
                    state: Err("no value supplied for state".to_string()),
                }
            }
        }

        impl Instance {
            pub fn disks<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::DiskAttachment>>,
                T::Error: ::std::fmt::Display,
            {
                self.disks = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for disks: {e}"));
                self
            }
            pub fn nics<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::NetworkInterface>>,
                T::Error: ::std::fmt::Display,
            {
                self.nics = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for nics: {e}"));
                self
            }
            pub fn properties<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::InstanceProperties>,
                T::Error: ::std::fmt::Display,
            {
                self.properties = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for properties: {e}"));
                self
            }
            pub fn state<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::InstanceState>,
                T::Error: ::std::fmt::Display,
            {
                self.state = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for state: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<Instance> for super::Instance {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Instance,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    disks: value.disks?,
                    nics: value.nics?,
                    properties: value.properties?,
                    state: value.state?,
                })
            }
        }

        impl ::std::convert::From<super::Instance> for Instance {
            fn from(value: super::Instance) -> Self {
                Self {
                    disks: Ok(value.disks),
                    nics: Ok(value.nics),
                    properties: Ok(value.properties),
                    state: Ok(value.state),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct InstanceEnsureRequest {
            cloud_init_bytes: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            disks:
                ::std::result::Result<::std::vec::Vec<super::DiskRequest>, ::std::string::String>,
            migrate: ::std::result::Result<
                ::std::option::Option<super::InstanceMigrateInitiateRequest>,
                ::std::string::String,
            >,
            nics: ::std::result::Result<
                ::std::vec::Vec<super::NetworkInterfaceRequest>,
                ::std::string::String,
            >,
            properties: ::std::result::Result<super::InstanceProperties, ::std::string::String>,
        }

        impl ::std::default::Default for InstanceEnsureRequest {
            fn default() -> Self {
                Self {
                    cloud_init_bytes: Ok(::std::default::Default::default()),
                    disks: Ok(::std::default::Default::default()),
                    migrate: Ok(::std::default::Default::default()),
                    nics: Ok(::std::default::Default::default()),
                    properties: Err("no value supplied for properties".to_string()),
                }
            }
        }

        impl InstanceEnsureRequest {
            pub fn cloud_init_bytes<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.cloud_init_bytes = value.try_into().map_err(|e| {
                    format!("error converting supplied value for cloud_init_bytes: {e}")
                });
                self
            }
            pub fn disks<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::DiskRequest>>,
                T::Error: ::std::fmt::Display,
            {
                self.disks = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for disks: {e}"));
                self
            }
            pub fn migrate<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                    ::std::option::Option<super::InstanceMigrateInitiateRequest>,
                >,
                T::Error: ::std::fmt::Display,
            {
                self.migrate = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for migrate: {e}"));
                self
            }
            pub fn nics<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::NetworkInterfaceRequest>>,
                T::Error: ::std::fmt::Display,
            {
                self.nics = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for nics: {e}"));
                self
            }
            pub fn properties<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::InstanceProperties>,
                T::Error: ::std::fmt::Display,
            {
                self.properties = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for properties: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<InstanceEnsureRequest> for super::InstanceEnsureRequest {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InstanceEnsureRequest,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    cloud_init_bytes: value.cloud_init_bytes?,
                    disks: value.disks?,
                    migrate: value.migrate?,
                    nics: value.nics?,
                    properties: value.properties?,
                })
            }
        }

        impl ::std::convert::From<super::InstanceEnsureRequest> for InstanceEnsureRequest {
            fn from(value: super::InstanceEnsureRequest) -> Self {
                Self {
                    cloud_init_bytes: Ok(value.cloud_init_bytes),
                    disks: Ok(value.disks),
                    migrate: Ok(value.migrate),
                    nics: Ok(value.nics),
                    properties: Ok(value.properties),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct InstanceEnsureResponse {
            migrate: ::std::result::Result<
                ::std::option::Option<super::InstanceMigrateInitiateResponse>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for InstanceEnsureResponse {
            fn default() -> Self {
                Self {
                    migrate: Ok(::std::default::Default::default()),
                }
            }
        }

        impl InstanceEnsureResponse {
            pub fn migrate<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                    ::std::option::Option<super::InstanceMigrateInitiateResponse>,
                >,
                T::Error: ::std::fmt::Display,
            {
                self.migrate = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for migrate: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<InstanceEnsureResponse> for super::InstanceEnsureResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InstanceEnsureResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    migrate: value.migrate?,
                })
            }
        }

        impl ::std::convert::From<super::InstanceEnsureResponse> for InstanceEnsureResponse {
            fn from(value: super::InstanceEnsureResponse) -> Self {
                Self {
                    migrate: Ok(value.migrate),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct InstanceGetResponse {
            instance: ::std::result::Result<super::Instance, ::std::string::String>,
        }

        impl ::std::default::Default for InstanceGetResponse {
            fn default() -> Self {
                Self {
                    instance: Err("no value supplied for instance".to_string()),
                }
            }
        }

        impl InstanceGetResponse {
            pub fn instance<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Instance>,
                T::Error: ::std::fmt::Display,
            {
                self.instance = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for instance: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<InstanceGetResponse> for super::InstanceGetResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InstanceGetResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    instance: value.instance?,
                })
            }
        }

        impl ::std::convert::From<super::InstanceGetResponse> for InstanceGetResponse {
            fn from(value: super::InstanceGetResponse) -> Self {
                Self {
                    instance: Ok(value.instance),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct InstanceMigrateInitiateRequest {
            migration_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
            src_addr: ::std::result::Result<::std::string::String, ::std::string::String>,
            src_uuid: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        }

        impl ::std::default::Default for InstanceMigrateInitiateRequest {
            fn default() -> Self {
                Self {
                    migration_id: Err("no value supplied for migration_id".to_string()),
                    src_addr: Err("no value supplied for src_addr".to_string()),
                    src_uuid: Err("no value supplied for src_uuid".to_string()),
                }
            }
        }

        impl InstanceMigrateInitiateRequest {
            pub fn migration_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::uuid::Uuid>,
                T::Error: ::std::fmt::Display,
            {
                self.migration_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for migration_id: {e}"));
                self
            }
            pub fn src_addr<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.src_addr = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for src_addr: {e}"));
                self
            }
            pub fn src_uuid<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::uuid::Uuid>,
                T::Error: ::std::fmt::Display,
            {
                self.src_uuid = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for src_uuid: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<InstanceMigrateInitiateRequest>
            for super::InstanceMigrateInitiateRequest
        {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InstanceMigrateInitiateRequest,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    migration_id: value.migration_id?,
                    src_addr: value.src_addr?,
                    src_uuid: value.src_uuid?,
                })
            }
        }

        impl ::std::convert::From<super::InstanceMigrateInitiateRequest>
            for InstanceMigrateInitiateRequest
        {
            fn from(value: super::InstanceMigrateInitiateRequest) -> Self {
                Self {
                    migration_id: Ok(value.migration_id),
                    src_addr: Ok(value.src_addr),
                    src_uuid: Ok(value.src_uuid),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct InstanceMigrateInitiateResponse {
            migration_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        }

        impl ::std::default::Default for InstanceMigrateInitiateResponse {
            fn default() -> Self {
                Self {
                    migration_id: Err("no value supplied for migration_id".to_string()),
                }
            }
        }

        impl InstanceMigrateInitiateResponse {
            pub fn migration_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::uuid::Uuid>,
                T::Error: ::std::fmt::Display,
            {
                self.migration_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for migration_id: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<InstanceMigrateInitiateResponse>
            for super::InstanceMigrateInitiateResponse
        {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InstanceMigrateInitiateResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    migration_id: value.migration_id?,
                })
            }
        }

        impl ::std::convert::From<super::InstanceMigrateInitiateResponse>
            for InstanceMigrateInitiateResponse
        {
            fn from(value: super::InstanceMigrateInitiateResponse) -> Self {
                Self {
                    migration_id: Ok(value.migration_id),
                }
            }
        }

        pub use self::InstanceMigrateInitiateResponse as InstanceMigrateStatusRequest;
        #[derive(Clone, Debug)]
        pub struct InstanceMigrateStatusResponse {
            state: ::std::result::Result<super::MigrationState, ::std::string::String>,
        }

        impl ::std::default::Default for InstanceMigrateStatusResponse {
            fn default() -> Self {
                Self {
                    state: Err("no value supplied for state".to_string()),
                }
            }
        }

        impl InstanceMigrateStatusResponse {
            pub fn state<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::MigrationState>,
                T::Error: ::std::fmt::Display,
            {
                self.state = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for state: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<InstanceMigrateStatusResponse>
            for super::InstanceMigrateStatusResponse
        {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InstanceMigrateStatusResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    state: value.state?,
                })
            }
        }

        impl ::std::convert::From<super::InstanceMigrateStatusResponse> for InstanceMigrateStatusResponse {
            fn from(value: super::InstanceMigrateStatusResponse) -> Self {
                Self {
                    state: Ok(value.state),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct InstanceProperties {
            bootrom_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
            description: ::std::result::Result<::std::string::String, ::std::string::String>,
            id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
            image_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
            memory: ::std::result::Result<u64, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            vcpus: ::std::result::Result<u8, ::std::string::String>,
        }

        impl ::std::default::Default for InstanceProperties {
            fn default() -> Self {
                Self {
                    bootrom_id: Err("no value supplied for bootrom_id".to_string()),
                    description: Err("no value supplied for description".to_string()),
                    id: Err("no value supplied for id".to_string()),
                    image_id: Err("no value supplied for image_id".to_string()),
                    memory: Err("no value supplied for memory".to_string()),
                    name: Err("no value supplied for name".to_string()),
                    vcpus: Err("no value supplied for vcpus".to_string()),
                }
            }
        }

        impl InstanceProperties {
            pub fn bootrom_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::uuid::Uuid>,
                T::Error: ::std::fmt::Display,
            {
                self.bootrom_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for bootrom_id: {e}"));
                self
            }
            pub fn description<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.description = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for description: {e}"));
                self
            }
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::uuid::Uuid>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
                self
            }
            pub fn image_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::uuid::Uuid>,
                T::Error: ::std::fmt::Display,
            {
                self.image_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for image_id: {e}"));
                self
            }
            pub fn memory<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u64>,
                T::Error: ::std::fmt::Display,
            {
                self.memory = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for memory: {e}"));
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
            pub fn vcpus<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u8>,
                T::Error: ::std::fmt::Display,
            {
                self.vcpus = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for vcpus: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<InstanceProperties> for super::InstanceProperties {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InstanceProperties,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    bootrom_id: value.bootrom_id?,
                    description: value.description?,
                    id: value.id?,
                    image_id: value.image_id?,
                    memory: value.memory?,
                    name: value.name?,
                    vcpus: value.vcpus?,
                })
            }
        }

        impl ::std::convert::From<super::InstanceProperties> for InstanceProperties {
            fn from(value: super::InstanceProperties) -> Self {
                Self {
                    bootrom_id: Ok(value.bootrom_id),
                    description: Ok(value.description),
                    id: Ok(value.id),
                    image_id: Ok(value.image_id),
                    memory: Ok(value.memory),
                    name: Ok(value.name),
                    vcpus: Ok(value.vcpus),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct InstanceStateMonitorRequest {
            gen_: ::std::result::Result<u64, ::std::string::String>,
        }

        impl ::std::default::Default for InstanceStateMonitorRequest {
            fn default() -> Self {
                Self {
                    gen_: Err("no value supplied for gen_".to_string()),
                }
            }
        }

        impl InstanceStateMonitorRequest {
            pub fn gen_<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u64>,
                T::Error: ::std::fmt::Display,
            {
                self.gen_ = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for gen_: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<InstanceStateMonitorRequest> for super::InstanceStateMonitorRequest {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InstanceStateMonitorRequest,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self { gen_: value.gen_? })
            }
        }

        impl ::std::convert::From<super::InstanceStateMonitorRequest> for InstanceStateMonitorRequest {
            fn from(value: super::InstanceStateMonitorRequest) -> Self {
                Self {
                    gen_: Ok(value.gen_),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct InstanceStateMonitorResponse {
            gen_: ::std::result::Result<u64, ::std::string::String>,
            state: ::std::result::Result<super::InstanceState, ::std::string::String>,
        }

        impl ::std::default::Default for InstanceStateMonitorResponse {
            fn default() -> Self {
                Self {
                    gen_: Err("no value supplied for gen_".to_string()),
                    state: Err("no value supplied for state".to_string()),
                }
            }
        }

        impl InstanceStateMonitorResponse {
            pub fn gen_<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u64>,
                T::Error: ::std::fmt::Display,
            {
                self.gen_ = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for gen_: {e}"));
                self
            }
            pub fn state<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::InstanceState>,
                T::Error: ::std::fmt::Display,
            {
                self.state = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for state: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<InstanceStateMonitorResponse> for super::InstanceStateMonitorResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InstanceStateMonitorResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    gen_: value.gen_?,
                    state: value.state?,
                })
            }
        }

        impl ::std::convert::From<super::InstanceStateMonitorResponse> for InstanceStateMonitorResponse {
            fn from(value: super::InstanceStateMonitorResponse) -> Self {
                Self {
                    gen_: Ok(value.gen_),
                    state: Ok(value.state),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct NetworkInterface {
            attachment: ::std::result::Result<
                super::NetworkInterfaceAttachmentState,
                ::std::string::String,
            >,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for NetworkInterface {
            fn default() -> Self {
                Self {
                    attachment: Err("no value supplied for attachment".to_string()),
                    name: Err("no value supplied for name".to_string()),
                }
            }
        }

        impl NetworkInterface {
            pub fn attachment<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::NetworkInterfaceAttachmentState>,
                T::Error: ::std::fmt::Display,
            {
                self.attachment = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for attachment: {e}"));
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

        impl ::std::convert::TryFrom<NetworkInterface> for super::NetworkInterface {
            type Error = super::error::ConversionError;
            fn try_from(
                value: NetworkInterface,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    attachment: value.attachment?,
                    name: value.name?,
                })
            }
        }

        impl ::std::convert::From<super::NetworkInterface> for NetworkInterface {
            fn from(value: super::NetworkInterface) -> Self {
                Self {
                    attachment: Ok(value.attachment),
                    name: Ok(value.name),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct NetworkInterfaceRequest {
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            slot: ::std::result::Result<super::Slot, ::std::string::String>,
        }

        impl ::std::default::Default for NetworkInterfaceRequest {
            fn default() -> Self {
                Self {
                    name: Err("no value supplied for name".to_string()),
                    slot: Err("no value supplied for slot".to_string()),
                }
            }
        }

        impl NetworkInterfaceRequest {
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
            pub fn slot<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Slot>,
                T::Error: ::std::fmt::Display,
            {
                self.slot = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for slot: {e}"));
                self
            }
        }

        impl ::std::convert::TryFrom<NetworkInterfaceRequest> for super::NetworkInterfaceRequest {
            type Error = super::error::ConversionError;
            fn try_from(
                value: NetworkInterfaceRequest,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    name: value.name?,
                    slot: value.slot?,
                })
            }
        }

        impl ::std::convert::From<super::NetworkInterfaceRequest> for NetworkInterfaceRequest {
            fn from(value: super::NetworkInterfaceRequest) -> Self {
                Self {
                    name: Ok(value.name),
                    slot: Ok(value.slot),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
///Client for Oxide Propolis Server API
///
///API for interacting with the Propolis hypervisor frontend.
///
///Version: 0.0.1
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
        "0.0.1"
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
    ///Sends a `GET` request to `/instance`
    ///
    ///```ignore
    /// let response = client.instance_get()
    ///    .send()
    ///    .await;
    /// ```
    pub fn instance_get(&self) -> builder::InstanceGet<'_> {
        builder::InstanceGet::new(self)
    }

    ///Sends a `PUT` request to `/instance`
    ///
    ///```ignore
    /// let response = client.instance_ensure()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn instance_ensure(&self) -> builder::InstanceEnsure<'_> {
        builder::InstanceEnsure::new(self)
    }

    ///Issue a snapshot request to a crucible backend
    ///
    ///Sends a `POST` request to `/instance/disk/{id}/snapshot/{snapshot_id}`
    ///
    ///```ignore
    /// let response = client.instance_issue_crucible_snapshot_request()
    ///    .id(id)
    ///    .snapshot_id(snapshot_id)
    ///    .send()
    ///    .await;
    /// ```
    pub fn instance_issue_crucible_snapshot_request(
        &self,
    ) -> builder::InstanceIssueCrucibleSnapshotRequest<'_> {
        builder::InstanceIssueCrucibleSnapshotRequest::new(self)
    }

    ///Sends a `GET` request to `/instance/migrate/status`
    ///
    ///```ignore
    /// let response = client.instance_migrate_status()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn instance_migrate_status(&self) -> builder::InstanceMigrateStatus<'_> {
        builder::InstanceMigrateStatus::new(self)
    }

    ///Sends a `GET` request to `/instance/serial`
    ///
    ///```ignore
    /// let response = client.instance_serial()
    ///    .send()
    ///    .await;
    /// ```
    pub fn instance_serial(&self) -> builder::InstanceSerial<'_> {
        builder::InstanceSerial::new(self)
    }

    ///Sends a `PUT` request to `/instance/state`
    ///
    ///```ignore
    /// let response = client.instance_state_put()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn instance_state_put(&self) -> builder::InstanceStatePut<'_> {
        builder::InstanceStatePut::new(self)
    }

    ///Sends a `GET` request to `/instance/state-monitor`
    ///
    ///```ignore
    /// let response = client.instance_state_monitor()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn instance_state_monitor(&self) -> builder::InstanceStateMonitor<'_> {
        builder::InstanceStateMonitor::new(self)
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
    ///Builder for [`Client::instance_get`]
    ///
    ///[`Client::instance_get`]: super::Client::instance_get
    #[derive(Debug, Clone)]
    pub struct InstanceGet<'a> {
        client: &'a super::Client,
    }

    impl<'a> InstanceGet<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/instance`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::InstanceGetResponse>, Error<types::Error>> {
            let Self { client } = self;
            let url = format!("{}/instance", client.baseurl,);
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
                operation_id: "instance_get",
            };
            client
                .__progenitor_response(
                    request,
                    &info,
                    &[(200u16, 200u16)],
                    &[(400u16, 499u16), (500u16, 599u16)],
                )
                .await
        }
    }

    ///Builder for [`Client::instance_ensure`]
    ///
    ///[`Client::instance_ensure`]: super::Client::instance_ensure
    #[derive(Debug, Clone)]
    pub struct InstanceEnsure<'a> {
        client: &'a super::Client,
        body: ::std::result::Result<types::builder::InstanceEnsureRequest, ::std::string::String>,
    }

    impl<'a> InstanceEnsure<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::InstanceEnsureRequest>,
            <V as std::convert::TryInto<types::InstanceEnsureRequest>>::Error: std::fmt::Display,
        {
            self.body = value.try_into().map(From::from).map_err(|s| {
                format!(
                    "conversion to `InstanceEnsureRequest` for body failed: {}",
                    s
                )
            });
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(
                types::builder::InstanceEnsureRequest,
            ) -> types::builder::InstanceEnsureRequest,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `PUT` request to `/instance`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::InstanceEnsureResponse>, Error<types::Error>> {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::InstanceEnsureRequest::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/instance", client.baseurl,);
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
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "instance_ensure",
            };
            client
                .__progenitor_response(
                    request,
                    &info,
                    &[(201u16, 201u16)],
                    &[(400u16, 499u16), (500u16, 599u16)],
                )
                .await
        }
    }

    ///Builder for [`Client::instance_issue_crucible_snapshot_request`]
    ///
    ///[`Client::instance_issue_crucible_snapshot_request`]: super::Client::instance_issue_crucible_snapshot_request
    #[derive(Debug, Clone)]
    pub struct InstanceIssueCrucibleSnapshotRequest<'a> {
        client: &'a super::Client,
        id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        snapshot_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
    }

    impl<'a> InstanceIssueCrucibleSnapshotRequest<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                id: Err("id was not initialized".to_string()),
                snapshot_id: Err("snapshot_id was not initialized".to_string()),
            }
        }

        pub fn id<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::uuid::Uuid>,
        {
            self.id = value
                .try_into()
                .map_err(|_| "conversion to `:: uuid :: Uuid` for id failed".to_string());
            self
        }

        pub fn snapshot_id<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::uuid::Uuid>,
        {
            self.snapshot_id = value
                .try_into()
                .map_err(|_| "conversion to `:: uuid :: Uuid` for snapshot_id failed".to_string());
            self
        }

        ///Sends a `POST` request to
        /// `/instance/disk/{id}/snapshot/{snapshot_id}`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<types::Error>> {
            let Self {
                client,
                id,
                snapshot_id,
            } = self;
            let id = id.map_err(Error::InvalidRequest)?;
            let snapshot_id = snapshot_id.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/instance/disk/{}/snapshot/{}",
                client.baseurl,
                encode_path(&id.to_string()),
                encode_path(&snapshot_id.to_string()),
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
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "instance_issue_crucible_snapshot_request",
            };
            client
                .__progenitor_response(
                    request,
                    &info,
                    &[(200u16, 200u16)],
                    &[(400u16, 499u16), (500u16, 599u16)],
                )
                .await
        }
    }

    ///Builder for [`Client::instance_migrate_status`]
    ///
    ///[`Client::instance_migrate_status`]: super::Client::instance_migrate_status
    #[derive(Debug, Clone)]
    pub struct InstanceMigrateStatus<'a> {
        client: &'a super::Client,
        body: ::std::result::Result<
            types::builder::InstanceMigrateStatusRequest,
            ::std::string::String,
        >,
    }

    impl<'a> InstanceMigrateStatus<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::InstanceMigrateStatusRequest>,
            <V as std::convert::TryInto<types::InstanceMigrateStatusRequest>>::Error:
                std::fmt::Display,
        {
            self.body = value.try_into().map(From::from).map_err(|s| {
                format!(
                    "conversion to `InstanceMigrateStatusRequest` for body failed: {}",
                    s
                )
            });
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(
                types::builder::InstanceMigrateStatusRequest,
            ) -> types::builder::InstanceMigrateStatusRequest,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `GET` request to `/instance/migrate/status`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::InstanceMigrateStatusResponse>, Error<types::Error>>
        {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| {
                    types::InstanceMigrateStatusRequest::try_from(v).map_err(|e| e.to_string())
                })
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/instance/migrate/status", client.baseurl,);
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
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "instance_migrate_status",
            };
            client
                .__progenitor_response(
                    request,
                    &info,
                    &[(200u16, 200u16)],
                    &[(400u16, 499u16), (500u16, 599u16)],
                )
                .await
        }
    }

    ///Builder for [`Client::instance_serial`]
    ///
    ///[`Client::instance_serial`]: super::Client::instance_serial
    #[derive(Debug, Clone)]
    pub struct InstanceSerial<'a> {
        client: &'a super::Client,
    }

    impl<'a> InstanceSerial<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/instance/serial`
        pub async fn send(self) -> Result<ResponseValue<reqwest::Upgraded>, Error<types::Error>> {
            let Self { client } = self;
            let url = format!("{}/instance/serial", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .headers(header_map)
                .header(::reqwest::header::CONNECTION, "Upgrade")
                .header(::reqwest::header::UPGRADE, "websocket")
                .header(::reqwest::header::SEC_WEBSOCKET_VERSION, "13")
                .header(
                    ::reqwest::header::SEC_WEBSOCKET_KEY,
                    ::base64::Engine::encode(
                        &::base64::engine::general_purpose::STANDARD,
                        ::rand::random::<[u8; 16]>(),
                    ),
                )
                .build()?;
            let info = OperationInfo {
                operation_id: "instance_serial",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                101u16 => ResponseValue::upgrade(response).await,
                400u16..=499u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16..=599u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::instance_state_put`]
    ///
    ///[`Client::instance_state_put`]: super::Client::instance_state_put
    #[derive(Debug, Clone)]
    pub struct InstanceStatePut<'a> {
        client: &'a super::Client,
        body: ::std::result::Result<types::InstanceStateRequested, ::std::string::String>,
    }

    impl<'a> InstanceStatePut<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Err("body was not initialized".to_string()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::InstanceStateRequested>,
        {
            self.body = value
                .try_into()
                .map_err(|_| "conversion to `InstanceStateRequested` for body failed".to_string());
            self
        }

        ///Sends a `PUT` request to `/instance/state`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<types::Error>> {
            let Self { client, body } = self;
            let body = body.map_err(Error::InvalidRequest)?;
            let url = format!("{}/instance/state", client.baseurl,);
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
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "instance_state_put",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                204u16 => Ok(ResponseValue::empty(response)),
                400u16..=499u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16..=599u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::instance_state_monitor`]
    ///
    ///[`Client::instance_state_monitor`]: super::Client::instance_state_monitor
    #[derive(Debug, Clone)]
    pub struct InstanceStateMonitor<'a> {
        client: &'a super::Client,
        body: ::std::result::Result<
            types::builder::InstanceStateMonitorRequest,
            ::std::string::String,
        >,
    }

    impl<'a> InstanceStateMonitor<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::InstanceStateMonitorRequest>,
            <V as std::convert::TryInto<types::InstanceStateMonitorRequest>>::Error:
                std::fmt::Display,
        {
            self.body = value.try_into().map(From::from).map_err(|s| {
                format!(
                    "conversion to `InstanceStateMonitorRequest` for body failed: {}",
                    s
                )
            });
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(
                types::builder::InstanceStateMonitorRequest,
            ) -> types::builder::InstanceStateMonitorRequest,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `GET` request to `/instance/state-monitor`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::InstanceStateMonitorResponse>, Error<types::Error>>
        {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| {
                    types::InstanceStateMonitorRequest::try_from(v).map_err(|e| e.to_string())
                })
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/instance/state-monitor", client.baseurl,);
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
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "instance_state_monitor",
            };
            client
                .__progenitor_response(
                    request,
                    &info,
                    &[(200u16, 200u16)],
                    &[(400u16, 499u16), (500u16, 599u16)],
                )
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
