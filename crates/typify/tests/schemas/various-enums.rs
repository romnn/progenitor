#![deny(warnings)]
#[doc = r" Error types."]
pub mod error {
    #[doc = r" Error from a `TryFrom` or `FromStr` implementation."]
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
#[doc = "`AlternativeEnum`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"default\": \"Choice2\",\n  \"type\": \"string\",\n  \"enum\": [\n    \"Choice1\",\n    \"Choice2\",\n    \"Choice3\"\n  ]\n}\n ```\n </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum AlternativeEnum {
    Choice1,
    Choice2,
    Choice3,
}
impl ::std::fmt::Display for AlternativeEnum {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Choice1 => f.write_str("Choice1"),
            Self::Choice2 => f.write_str("Choice2"),
            Self::Choice3 => f.write_str("Choice3"),
        }
    }
}
impl ::std::str::FromStr for AlternativeEnum {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "Choice1" => Ok(Self::Choice1),
            "Choice2" => Ok(Self::Choice2),
            "Choice3" => Ok(Self::Choice3),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for AlternativeEnum {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for AlternativeEnum {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for AlternativeEnum {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::default::Default for AlternativeEnum {
    fn default() -> Self {
        AlternativeEnum::Choice2
    }
}
#[doc = "`AnyOfNoStrings`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\",\n  \"enum\": []\n}\n ```\n </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
#[serde(deny_unknown_fields)]
pub enum AnyOfNoStrings {}
#[doc = "`AnyOfNothing`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"enum\": []\n}\n ```\n </details>"]
pub use self::AnyOfNoStrings as AnyOfNothing;
#[doc = "`CommentedVariants`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"oneOf\": [\n    {\n      \"description\": \"An A\",\n      \"enum\": [\n        \"A\"\n      ]\n    },\n    {\n      \"description\": \"A B\",\n      \"enum\": [\n        \"B\"\n      ]\n    },\n    {\n      \"description\": \"a pirate's favorite letter\",\n      \"const\": \"C\"\n    }\n  ]\n}\n ```\n </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum CommentedVariants {
    #[doc = "An A"]
    A,
    #[doc = "A B"]
    B,
    #[doc = "a pirate's favorite letter"]
    C,
}
impl ::std::fmt::Display for CommentedVariants {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::A => f.write_str("A"),
            Self::B => f.write_str("B"),
            Self::C => f.write_str("C"),
        }
    }
}
impl ::std::str::FromStr for CommentedVariants {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CommentedVariants {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CommentedVariants {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CommentedVariants {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`DiskAttachment`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"required\": [\n    \"alternate\",\n    \"state\"\n  ],\n  \"properties\": {\n    \"alternate\": {\n      \"$ref\": \"#/components/schemas/AlternativeEnum\"\n    },\n    \"state\": {\n      \"default\": \"Detached\",\n      \"type\": \"string\",\n      \"enum\": [\n        \"Detached\",\n        \"Destroyed\",\n        \"Faulted\"\n      ]\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct DiskAttachment {
    pub alternate: AlternativeEnum,
    pub state: DiskAttachmentState,
}
impl DiskAttachment {
    pub fn builder() -> builder::DiskAttachment {
        ::std::default::Default::default()
    }
}
#[doc = "`DiskAttachmentState`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"default\": \"Detached\",\n  \"type\": \"string\",\n  \"enum\": [\n    \"Detached\",\n    \"Destroyed\",\n    \"Faulted\"\n  ]\n}\n ```\n </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum DiskAttachmentState {
    Detached,
    Destroyed,
    Faulted,
}
impl ::std::fmt::Display for DiskAttachmentState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Detached => f.write_str("Detached"),
            Self::Destroyed => f.write_str("Destroyed"),
            Self::Faulted => f.write_str("Faulted"),
        }
    }
}
impl ::std::str::FromStr for DiskAttachmentState {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "Detached" => Ok(Self::Detached),
            "Destroyed" => Ok(Self::Destroyed),
            "Faulted" => Ok(Self::Faulted),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for DiskAttachmentState {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DiskAttachmentState {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DiskAttachmentState {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::default::Default for DiskAttachmentState {
    fn default() -> Self {
        DiskAttachmentState::Detached
    }
}
#[doc = "`EmptyObject`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"properties\": {\n    \"prop\": {\n      \"type\": \"object\",\n      \"enum\": [\n        {}\n      ]\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct EmptyObject {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub prop: ::std::option::Option<EmptyObjectProp>,
}
impl ::std::default::Default for EmptyObject {
    fn default() -> Self {
        Self {
            prop: ::std::default::Default::default(),
        }
    }
}
impl EmptyObject {
    pub fn builder() -> builder::EmptyObject {
        ::std::default::Default::default()
    }
}
#[doc = "`EmptyObjectProp`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"enum\": [\n    {}\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct EmptyObjectProp(::serde_json::Map<::std::string::String, ::serde_json::Value>);
impl ::std::ops::Deref for EmptyObjectProp {
    type Target = ::serde_json::Map<::std::string::String, ::serde_json::Value>;
    fn deref(&self) -> &::serde_json::Map<::std::string::String, ::serde_json::Value> {
        &self.0
    }
}
impl ::std::convert::From<EmptyObjectProp>
    for ::serde_json::Map<::std::string::String, ::serde_json::Value>
{
    fn from(value: EmptyObjectProp) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<::serde_json::Map<::std::string::String, ::serde_json::Value>>
    for EmptyObjectProp
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![[].into_iter().collect()].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for EmptyObjectProp {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<::serde_json::Map<
            ::std::string::String,
            ::serde_json::Value,
        >>::deserialize(deserializer)?)
        .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`EnumAndConstant`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"oneOf\": [\n    {\n      \"type\": \"object\",\n      \"required\": [\n        \"bark\",\n        \"petType\"\n      ],\n      \"properties\": {\n        \"bark\": {\n          \"type\": \"string\"\n        },\n        \"petType\": {\n          \"type\": \"string\",\n          \"enum\": [\n            \"dog\"\n          ]\n        }\n      }\n    },\n    {\n      \"type\": \"object\",\n      \"required\": [\n        \"petType\",\n        \"purr\"\n      ],\n      \"properties\": {\n        \"petType\": {\n          \"type\": \"string\",\n          \"const\": \"cat\"\n        },\n        \"purr\": {\n          \"type\": \"string\"\n        }\n      }\n    },\n    {\n      \"type\": \"object\",\n      \"required\": [\n        \"help\",\n        \"petType\"\n      ],\n      \"properties\": {\n        \"help\": {\n          \"type\": \"string\"\n        },\n        \"petType\": {\n          \"const\": \"monkey\"\n        }\n      }\n    },\n    {\n      \"type\": \"object\",\n      \"required\": [\n        \"float\",\n        \"petType\"\n      ],\n      \"properties\": {\n        \"float\": {\n          \"type\": \"string\"\n        },\n        \"petType\": {\n          \"enum\": [\n            \"fish\"\n          ]\n        }\n      }\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(tag = "petType")]
pub enum EnumAndConstant {
    #[serde(rename = "dog")]
    Dog { bark: ::std::string::String },
    #[serde(rename = "cat")]
    Cat { purr: ::std::string::String },
    #[serde(rename = "monkey")]
    Monkey { help: ::std::string::String },
    #[serde(rename = "fish")]
    Fish { float: ::std::string::String },
}
#[doc = "`IpNet`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"oneOf\": [\n    {\n      \"title\": \"V4\",\n      \"allOf\": [\n        {\n          \"$ref\": \"#/components/schemas/Ipv4Net\"\n        }\n      ]\n    },\n    {\n      \"title\": \"V6\",\n      \"allOf\": [\n        {\n          \"$ref\": \"#/components/schemas/Ipv6Net\"\n        }\n      ]\n    }\n  ],\n  \"$comment\": \"we want to see *nice* variant names in the output\"\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum IpNet {
    V4(Ipv4Net),
    V6(Ipv6Net),
}
impl ::std::str::FromStr for IpNet {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if let Ok(v) = value.parse() {
            Ok(Self::V4(v))
        } else if let Ok(v) = value.parse() {
            Ok(Self::V6(v))
        } else {
            Err("string conversion failed for all variants".into())
        }
    }
}
impl ::std::convert::TryFrom<&str> for IpNet {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for IpNet {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for IpNet {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::fmt::Display for IpNet {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::V4(x) => x.fmt(f),
            Self::V6(x) => x.fmt(f),
        }
    }
}
impl ::std::convert::From<Ipv4Net> for IpNet {
    fn from(value: Ipv4Net) -> Self {
        Self::V4(value)
    }
}
impl ::std::convert::From<Ipv6Net> for IpNet {
    fn from(value: Ipv6Net) -> Self {
        Self::V6(value)
    }
}
#[doc = "`Ipv4Net`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\",\n  \"pattern\": \".*\"\n}\n ```\n </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Ipv4Net(::std::string::String);
impl ::std::ops::Deref for Ipv4Net {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<Ipv4Net> for ::std::string::String {
    fn from(value: Ipv4Net) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for Ipv4Net {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new(".*").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \".*\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for Ipv4Net {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Ipv4Net {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Ipv4Net {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for Ipv4Net {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`Ipv6Net`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\",\n  \"pattern\": \".*\"\n}\n ```\n </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Ipv6Net(::std::string::String);
impl ::std::ops::Deref for Ipv6Net {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<Ipv6Net> for ::std::string::String {
    fn from(value: Ipv6Net) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for Ipv6Net {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new(".*").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \".*\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for Ipv6Net {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Ipv6Net {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Ipv6Net {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for Ipv6Net {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`JankNames`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"oneOf\": [\n    {\n      \"title\": \"Animation Specification\",\n      \"type\": \"string\"\n    },\n    {\n      \"title\": \"Animation Specification\",\n      \"type\": \"object\",\n      \"maxProperties\": 1,\n      \"minProperties\": 1,\n      \"additionalProperties\": {\n        \"type\": \"string\"\n      }\n    },\n    {\n      \"type\": \"object\",\n      \"maxProperties\": 2,\n      \"minProperties\": 2,\n      \"additionalProperties\": {\n        \"type\": \"integer\"\n      }\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum JankNames {
    Variant0(::std::string::String),
    Variant1(::std::collections::HashMap<::std::string::String, ::std::string::String>),
    Variant2(::std::collections::HashMap<::std::string::String, i64>),
}
impl ::std::convert::From<::std::collections::HashMap<::std::string::String, ::std::string::String>>
    for JankNames
{
    fn from(
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) -> Self {
        Self::Variant1(value)
    }
}
impl ::std::convert::From<::std::collections::HashMap<::std::string::String, i64>> for JankNames {
    fn from(value: ::std::collections::HashMap<::std::string::String, i64>) -> Self {
        Self::Variant2(value)
    }
}
#[doc = "`Never`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\nfalse\n ```\n </details>"]
pub use self::AnyOfNoStrings as Never;
#[doc = "`NeverEver`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\nfalse\n ```\n </details>"]
pub use self::AnyOfNoStrings as NeverEver;
#[doc = "`NeverEverForever`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\nfalse\n ```\n </details>"]
pub use self::AnyOfNoStrings as NeverEverForever;
#[doc = "`NullStringEnumWithUnknownFormat`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": [\n    \"string\",\n    \"null\"\n  ],\n  \"format\": \"?\",\n  \"enum\": [\n    \"a\",\n    \"b\",\n    \"c\"\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct NullStringEnumWithUnknownFormat(
    pub ::std::option::Option<NullStringEnumWithUnknownFormatInner>,
);
impl ::std::ops::Deref for NullStringEnumWithUnknownFormat {
    type Target = ::std::option::Option<NullStringEnumWithUnknownFormatInner>;
    fn deref(&self) -> &::std::option::Option<NullStringEnumWithUnknownFormatInner> {
        &self.0
    }
}
impl ::std::convert::From<NullStringEnumWithUnknownFormat>
    for ::std::option::Option<NullStringEnumWithUnknownFormatInner>
{
    fn from(value: NullStringEnumWithUnknownFormat) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::option::Option<NullStringEnumWithUnknownFormatInner>>
    for NullStringEnumWithUnknownFormat
{
    fn from(value: ::std::option::Option<NullStringEnumWithUnknownFormatInner>) -> Self {
        Self(value)
    }
}
#[doc = "`NullStringEnumWithUnknownFormatInner`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\",\n  \"format\": \"?\",\n  \"enum\": [\n    \"a\",\n    \"b\",\n    \"c\"\n  ]\n}\n ```\n </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum NullStringEnumWithUnknownFormatInner {
    #[serde(rename = "a")]
    A,
    #[serde(rename = "b")]
    B,
    #[serde(rename = "c")]
    C,
}
impl ::std::fmt::Display for NullStringEnumWithUnknownFormatInner {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::A => f.write_str("a"),
            Self::B => f.write_str("b"),
            Self::C => f.write_str("c"),
        }
    }
}
impl ::std::str::FromStr for NullStringEnumWithUnknownFormatInner {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            "c" => Ok(Self::C),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for NullStringEnumWithUnknownFormatInner {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for NullStringEnumWithUnknownFormatInner {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for NullStringEnumWithUnknownFormatInner {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`OneOfMissingTitle`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"oneOf\": [\n    {\n      \"title\": \"A\",\n      \"properties\": {\n        \"foo\": {\n          \"type\": \"string\"\n        }\n      }\n    },\n    {\n      \"title\": \"B\",\n      \"properties\": {\n        \"bar\": {\n          \"type\": \"integer\"\n        }\n      }\n    },\n    {\n      \"properties\": {\n        \"bar\": {\n          \"type\": \"integer\"\n        },\n        \"baz\": {\n          \"type\": \"integer\"\n        }\n      }\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum OneOfMissingTitle {
    Variant0 {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        foo: ::std::option::Option<::std::string::String>,
    },
    Variant1 {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        bar: ::std::option::Option<i64>,
    },
    Variant2 {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        bar: ::std::option::Option<i64>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        baz: ::std::option::Option<i64>,
    },
}
#[doc = "`OneOfRawType`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"oneOf\": [\n    {\n      \"type\": \"string\"\n    },\n    {\n      \"type\": \"integer\"\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum OneOfRawType {
    String(::std::string::String),
    Integer(i64),
}
impl ::std::fmt::Display for OneOfRawType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::String(x) => x.fmt(f),
            Self::Integer(x) => x.fmt(f),
        }
    }
}
impl ::std::convert::From<i64> for OneOfRawType {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}
#[doc = "`OneOfTypes`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"oneOf\": [\n    {\n      \"required\": [\n        \"bar\"\n      ],\n      \"properties\": {\n        \"bar\": {\n          \"type\": \"integer\"\n        }\n      }\n    },\n    {\n      \"required\": [\n        \"foo\"\n      ],\n      \"properties\": {\n        \"foo\": {\n          \"type\": \"string\"\n        }\n      }\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub enum OneOfTypes {
    #[serde(rename = "bar")]
    Bar(i64),
    #[serde(rename = "foo")]
    Foo(::std::string::String),
}
impl ::std::convert::From<i64> for OneOfTypes {
    fn from(value: i64) -> Self {
        Self::Bar(value)
    }
}
#[doc = "`OptionAnyofConst`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"anyOf\": [\n    {\n      \"type\": \"string\"\n    },\n    {\n      \"const\": null\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct OptionAnyofConst(pub ::std::option::Option<::std::string::String>);
impl ::std::ops::Deref for OptionAnyofConst {
    type Target = ::std::option::Option<::std::string::String>;
    fn deref(&self) -> &::std::option::Option<::std::string::String> {
        &self.0
    }
}
impl ::std::convert::From<OptionAnyofConst> for ::std::option::Option<::std::string::String> {
    fn from(value: OptionAnyofConst) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::option::Option<::std::string::String>> for OptionAnyofConst {
    fn from(value: ::std::option::Option<::std::string::String>) -> Self {
        Self(value)
    }
}
#[doc = "`OptionAnyofEnum`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"anyOf\": [\n    {\n      \"type\": \"string\"\n    },\n    {\n      \"enum\": [\n        null\n      ]\n    }\n  ]\n}\n ```\n </details>"]
pub use self::OptionAnyofConst as OptionAnyofEnum;
#[doc = "`OptionAnyofNull`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"anyOf\": [\n    {\n      \"type\": \"string\"\n    },\n    {\n      \"type\": \"null\"\n    }\n  ]\n}\n ```\n </details>"]
pub use self::OptionAnyofConst as OptionAnyofNull;
#[doc = "`OptionOneofConst`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"oneOf\": [\n    {\n      \"type\": \"string\"\n    },\n    {\n      \"const\": null\n    }\n  ]\n}\n ```\n </details>"]
pub use self::OptionAnyofConst as OptionOneofConst;
#[doc = "`OptionOneofEnum`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"oneOf\": [\n    {\n      \"type\": \"string\"\n    },\n    {\n      \"enum\": [\n        null\n      ]\n    }\n  ]\n}\n ```\n </details>"]
pub use self::OptionAnyofConst as OptionOneofEnum;
#[doc = "`OptionOneofNull`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"oneOf\": [\n    {\n      \"type\": \"string\"\n    },\n    {\n      \"type\": \"null\"\n    }\n  ]\n}\n ```\n </details>"]
pub use self::OptionAnyofConst as OptionOneofNull;
#[doc = "`ReferenceDef`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\"\n}\n ```\n </details>"]
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
pub struct ReferenceDef(pub ::std::string::String);
impl ::std::ops::Deref for ReferenceDef {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ReferenceDef> for ::std::string::String {
    fn from(value: ReferenceDef) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::string::String> for ReferenceDef {
    fn from(value: ::std::string::String) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for ReferenceDef {
    type Err = ::std::convert::Infallible;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.to_string()))
    }
}
impl ::std::fmt::Display for ReferenceDef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}
#[doc = "issue 280"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"description\": \"issue 280\",\n  \"oneOf\": [\n    {\n      \"type\": \"array\",\n      \"items\": {\n        \"type\": \"string\"\n      }\n    },\n    {\n      \"type\": \"object\",\n      \"additionalProperties\": {\n        \"oneOf\": [\n          {\n            \"$ref\": \"#/definitions/StringVersion\"\n          },\n          {\n            \"$ref\": \"#/definitions/ReferenceDef\"\n          }\n        ]\n      },\n      \"$comment\": \"Mapping of mod name to the desired version\"\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum References {
    Array(::std::vec::Vec<::std::string::String>),
    Object(::std::collections::HashMap<::std::string::String, ReferencesObjectValue>),
}
impl ::std::convert::From<::std::vec::Vec<::std::string::String>> for References {
    fn from(value: ::std::vec::Vec<::std::string::String>) -> Self {
        Self::Array(value)
    }
}
impl ::std::convert::From<::std::collections::HashMap<::std::string::String, ReferencesObjectValue>>
    for References
{
    fn from(
        value: ::std::collections::HashMap<::std::string::String, ReferencesObjectValue>,
    ) -> Self {
        Self::Object(value)
    }
}
#[doc = "`ReferencesObjectValue`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"oneOf\": [\n    {\n      \"$ref\": \"#/definitions/StringVersion\"\n    },\n    {\n      \"$ref\": \"#/definitions/ReferenceDef\"\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ReferencesObjectValue {
    StringVersion(StringVersion),
    ReferenceDef(ReferenceDef),
}
impl ::std::fmt::Display for ReferencesObjectValue {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::StringVersion(x) => x.fmt(f),
            Self::ReferenceDef(x) => x.fmt(f),
        }
    }
}
impl ::std::convert::From<StringVersion> for ReferencesObjectValue {
    fn from(value: StringVersion) -> Self {
        Self::StringVersion(value)
    }
}
impl ::std::convert::From<ReferenceDef> for ReferencesObjectValue {
    fn from(value: ReferenceDef) -> Self {
        Self::ReferenceDef(value)
    }
}
#[doc = "`ShouldBeExclusive`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"oneOf\": [\n    {\n      \"required\": [\n        \"id\"\n      ]\n    },\n    {\n      \"required\": [\n        \"reference\"\n      ]\n    }\n  ],\n  \"properties\": {\n    \"id\": {\n      \"type\": \"string\"\n    },\n    \"reference\": {\n      \"type\": \"string\"\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ShouldBeExclusive {
    Variant0 { id: ::std::string::String },
    Variant1 { reference: ::std::string::String },
}
#[doc = "`StringVersion`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\"\n}\n ```\n </details>"]
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
pub struct StringVersion(pub ::std::string::String);
impl ::std::ops::Deref for StringVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StringVersion> for ::std::string::String {
    fn from(value: StringVersion) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::string::String> for StringVersion {
    fn from(value: ::std::string::String) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for StringVersion {
    type Err = ::std::convert::Infallible;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.to_string()))
    }
}
impl ::std::fmt::Display for StringVersion {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}
#[doc = "`VariantsDifferByPunct`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"enum\": [\n    \"2.5GBASE-T\",\n    \"25GBASE-T\",\n    \"2,5,GBASE,T\"\n  ]\n}\n ```\n </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum VariantsDifferByPunct {
    #[serde(rename = "2.5GBASE-T")]
    X2x5gbasext,
    #[serde(rename = "25GBASE-T")]
    X25gbasext,
    #[serde(rename = "2,5,GBASE,T")]
    X2x5xgbasext,
}
impl ::std::fmt::Display for VariantsDifferByPunct {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::X2x5gbasext => f.write_str("2.5GBASE-T"),
            Self::X25gbasext => f.write_str("25GBASE-T"),
            Self::X2x5xgbasext => f.write_str("2,5,GBASE,T"),
        }
    }
}
impl ::std::str::FromStr for VariantsDifferByPunct {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "2.5GBASE-T" => Ok(Self::X2x5gbasext),
            "25GBASE-T" => Ok(Self::X25gbasext),
            "2,5,GBASE,T" => Ok(Self::X2x5xgbasext),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for VariantsDifferByPunct {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for VariantsDifferByPunct {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for VariantsDifferByPunct {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct DiskAttachment {
        alternate: ::std::result::Result<super::AlternativeEnum, ::std::string::String>,
        state: ::std::result::Result<super::DiskAttachmentState, ::std::string::String>,
    }
    impl ::std::default::Default for DiskAttachment {
        fn default() -> Self {
            Self {
                alternate: Err("no value supplied for alternate".to_string()),
                state: Err("no value supplied for state".to_string()),
            }
        }
    }
    impl DiskAttachment {
        pub fn alternate<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AlternativeEnum>,
            T::Error: ::std::fmt::Display,
        {
            self.alternate = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for alternate: {e}"));
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
                alternate: value.alternate?,
                state: value.state?,
            })
        }
    }
    impl ::std::convert::From<super::DiskAttachment> for DiskAttachment {
        fn from(value: super::DiskAttachment) -> Self {
            Self {
                alternate: Ok(value.alternate),
                state: Ok(value.state),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EmptyObject {
        prop: ::std::result::Result<
            ::std::option::Option<super::EmptyObjectProp>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for EmptyObject {
        fn default() -> Self {
            Self {
                prop: Ok(::std::default::Default::default()),
            }
        }
    }
    impl EmptyObject {
        pub fn prop<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EmptyObjectProp>>,
            T::Error: ::std::fmt::Display,
        {
            self.prop = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prop: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EmptyObject> for super::EmptyObject {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EmptyObject,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { prop: value.prop? })
        }
    }
    impl ::std::convert::From<super::EmptyObject> for EmptyObject {
        fn from(value: super::EmptyObject) -> Self {
            Self {
                prop: Ok(value.prop),
            }
        }
    }
}
fn main() {}
