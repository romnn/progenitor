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
#[doc = "`BarProp`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"properties\": {\n    \"bar\": {\n      \"bar\": \"string\"\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct BarProp {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub bar: ::std::option::Option<::serde_json::Value>,
}
impl ::std::default::Default for BarProp {
    fn default() -> Self {
        Self {
            bar: ::std::default::Default::default(),
        }
    }
}
impl BarProp {
    pub fn builder() -> builder::BarProp {
        ::std::default::Default::default()
    }
}
#[doc = "`ButNotThat`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"not\": {\n    \"required\": [\n      \"that\"\n    ]\n  },\n  \"properties\": {\n    \"that\": {},\n    \"this\": {}\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct ButNotThat {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub this: ::std::option::Option<::serde_json::Value>,
}
impl ::std::default::Default for ButNotThat {
    fn default() -> Self {
        Self {
            this: ::std::default::Default::default(),
        }
    }
}
impl ButNotThat {
    pub fn builder() -> builder::ButNotThat {
        ::std::default::Default::default()
    }
}
#[doc = "if we don't see this, we dropped the metadata"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"description\": \"if we don't see this, we dropped the metadata\",\n  \"type\": \"object\",\n  \"allOf\": [\n    {\n      \"type\": \"object\",\n      \"properties\": {\n        \"y\": true\n      }\n    }\n  ],\n  \"properties\": {\n    \"x\": true\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct CommentedTypeMerged {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub x: ::std::option::Option<::serde_json::Value>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub y: ::std::option::Option<::serde_json::Value>,
}
impl ::std::default::Default for CommentedTypeMerged {
    fn default() -> Self {
        Self {
            x: ::std::default::Default::default(),
            y: ::std::default::Default::default(),
        }
    }
}
impl CommentedTypeMerged {
    pub fn builder() -> builder::CommentedTypeMerged {
        ::std::default::Default::default()
    }
}
#[doc = "`HereAndThere`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"type\": \"object\",\n      \"properties\": {\n        \"foo\": {\n          \"type\": \"string\"\n        }\n      }\n    }\n  ],\n  \"oneOf\": [\n    {\n      \"type\": \"object\",\n      \"properties\": {\n        \"bar\": {\n          \"type\": \"string\"\n        }\n      }\n    },\n    {\n      \"type\": \"object\",\n      \"properties\": {\n        \"baz\": {\n          \"type\": \"string\"\n        }\n      }\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum HereAndThere {
    Variant0 {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        bar: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        foo: ::std::option::Option<::std::string::String>,
    },
    Variant1 {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        baz: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        foo: ::std::option::Option<::std::string::String>,
    },
}
#[doc = "`JsonResponseBase`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"properties\": {\n    \"result\": {\n      \"type\": \"string\"\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct JsonResponseBase {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub result: ::std::option::Option<::std::string::String>,
}
impl ::std::default::Default for JsonResponseBase {
    fn default() -> Self {
        Self {
            result: ::std::default::Default::default(),
        }
    }
}
impl JsonResponseBase {
    pub fn builder() -> builder::JsonResponseBase {
        ::std::default::Default::default()
    }
}
#[doc = "`JsonSuccess`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"$ref\": \"#/components/schemas/JsonSuccessBase\"\n    },\n    {\n      \"properties\": {\n        \"msg\": {},\n        \"result\": {}\n      },\n      \"additionalProperties\": false\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct JsonSuccess {
    pub msg: ::std::string::String,
    pub result: JsonSuccessResult,
}
impl JsonSuccess {
    pub fn builder() -> builder::JsonSuccess {
        ::std::default::Default::default()
    }
}
#[doc = "x"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"description\": \"x\",\n  \"allOf\": [\n    {\n      \"$ref\": \"#/components/schemas/JsonResponseBase\"\n    },\n    {\n      \"required\": [\n        \"msg\",\n        \"result\"\n      ],\n      \"properties\": {\n        \"msg\": {\n          \"type\": \"string\"\n        },\n        \"result\": {\n          \"enum\": [\n            \"success\"\n          ]\n        }\n      }\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct JsonSuccessBase {
    pub msg: ::std::string::String,
    pub result: JsonSuccessBaseResult,
}
impl JsonSuccessBase {
    pub fn builder() -> builder::JsonSuccessBase {
        ::std::default::Default::default()
    }
}
#[doc = "`JsonSuccessBaseResult`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\",\n  \"enum\": [\n    \"success\"\n  ]\n}\n ```\n </details>"]
pub use self::JsonSuccessResult as JsonSuccessBaseResult;
#[doc = "`JsonSuccessResult`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\",\n  \"enum\": [\n    \"success\"\n  ]\n}\n ```\n </details>"]
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
pub enum JsonSuccessResult {
    #[serde(rename = "success")]
    Success,
}
impl ::std::fmt::Display for JsonSuccessResult {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Success => f.write_str("success"),
        }
    }
}
impl ::std::str::FromStr for JsonSuccessResult {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "success" => Ok(Self::Success),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for JsonSuccessResult {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JsonSuccessResult {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JsonSuccessResult {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MergeEmpty`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"type\": \"object\",\n      \"properties\": {\n        \"action\": {\n          \"type\": \"string\",\n          \"enum\": [\n            \"foo\"\n          ]\n        },\n        \"token\": {\n          \"type\": \"string\"\n        }\n      },\n      \"additionalProperties\": false\n    },\n    {\n      \"type\": \"object\",\n      \"properties\": {\n        \"action\": {\n          \"type\": \"string\",\n          \"enum\": [\n            \"bar\"\n          ]\n        }\n      },\n      \"additionalProperties\": false,\n      \"token\": {\n        \"type\": \"integer\"\n      }\n    }\n  ],\n  \"$comment\": \"properties conflict but are not required so we end up with an empty object\"\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MergeEmpty {}
impl ::std::default::Default for MergeEmpty {
    fn default() -> Self {
        Self {}
    }
}
impl MergeEmpty {
    pub fn builder() -> builder::MergeEmpty {
        ::std::default::Default::default()
    }
}
#[doc = "`MergeNumberBounds`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"type\": \"number\",\n      \"maximum\": 100.0,\n      \"minimum\": 1.0\n    },\n    {\n      \"type\": \"number\",\n      \"exclusiveMaximum\": 50.0,\n      \"exclusiveMinimum\": 5.0\n    }\n  ],\n  \"$comment\": \"merging number constraints takes the most restrictive bounds; mixed inclusive/exclusive bounds are normalized to drop the subsumed one\"\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct MergeNumberBounds(pub f64);
impl ::std::ops::Deref for MergeNumberBounds {
    type Target = f64;
    fn deref(&self) -> &f64 {
        &self.0
    }
}
impl ::std::convert::From<MergeNumberBounds> for f64 {
    fn from(value: MergeNumberBounds) -> Self {
        value.0
    }
}
impl ::std::convert::From<f64> for MergeNumberBounds {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for MergeNumberBounds {
    type Err = <f64 as ::std::str::FromStr>::Err;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}
impl ::std::convert::TryFrom<&str> for MergeNumberBounds {
    type Error = <f64 as ::std::str::FromStr>::Err;
    fn try_from(value: &str) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<String> for MergeNumberBounds {
    type Error = <f64 as ::std::str::FromStr>::Err;
    fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::fmt::Display for MergeNumberBounds {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}
#[doc = "`MergeStringBounds`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"type\": \"string\",\n      \"maxLength\": 20,\n      \"minLength\": 5\n    },\n    {\n      \"type\": \"string\",\n      \"maxLength\": 10,\n      \"minLength\": 2\n    }\n  ],\n  \"$comment\": \"merging string constraints takes the most restrictive bounds: max_length takes the min, min_length takes the max\"\n}\n ```\n </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MergeStringBounds(::std::string::String);
impl ::std::ops::Deref for MergeStringBounds {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MergeStringBounds> for ::std::string::String {
    fn from(value: MergeStringBounds) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MergeStringBounds {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 10usize {
            return Err("longer than 10 characters".into());
        }
        if value.chars().count() < 5usize {
            return Err("shorter than 5 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MergeStringBounds {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MergeStringBounds {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MergeStringBounds {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MergeStringBounds {
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
#[doc = "`NarrowNumber`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"type\": \"integer\"\n    },\n    {\n      \"minimum\": 1.0\n    }\n  ]\n}\n ```\n </details>"]
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
pub struct NarrowNumber(pub ::std::num::NonZeroU64);
impl ::std::ops::Deref for NarrowNumber {
    type Target = ::std::num::NonZeroU64;
    fn deref(&self) -> &::std::num::NonZeroU64 {
        &self.0
    }
}
impl ::std::convert::From<NarrowNumber> for ::std::num::NonZeroU64 {
    fn from(value: NarrowNumber) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::num::NonZeroU64> for NarrowNumber {
    fn from(value: ::std::num::NonZeroU64) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for NarrowNumber {
    type Err = <::std::num::NonZeroU64 as ::std::str::FromStr>::Err;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}
impl ::std::convert::TryFrom<&str> for NarrowNumber {
    type Error = <::std::num::NonZeroU64 as ::std::str::FromStr>::Err;
    fn try_from(value: &str) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<String> for NarrowNumber {
    type Error = <::std::num::NonZeroU64 as ::std::str::FromStr>::Err;
    fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::fmt::Display for NarrowNumber {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}
#[doc = "`OrderDependentMerge`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"$ref\": \"#/components/schemas/BarProp\"\n    },\n    {\n      \"properties\": {\n        \"baz\": {\n          \"type\": \"boolean\"\n        }\n      }\n    }\n  ],\n  \"required\": [\n    \"baz\"\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct OrderDependentMerge {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub bar: ::std::option::Option<::serde_json::Value>,
    pub baz: bool,
}
impl OrderDependentMerge {
    pub fn builder() -> builder::OrderDependentMerge {
        ::std::default::Default::default()
    }
}
#[doc = "`Pickingone`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"$ref\": \"#/definitions/pickingone-installation\"\n    },\n    {\n      \"type\": \"object\",\n      \"required\": [\n        \"suspended_by\"\n      ],\n      \"properties\": {\n        \"suspended_by\": {\n          \"$ref\": \"#/definitions/pickingone-user\"\n        }\n      }\n    }\n  ],\n  \"$comment\": \"TODO this generates an extra type for the pickingone-user dependency\"\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Pickingone {
    pub suspended_by: PickingoneSuspendedBy,
}
impl Pickingone {
    pub fn builder() -> builder::Pickingone {
        ::std::default::Default::default()
    }
}
#[doc = "`PickingoneInstallation`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"properties\": {\n    \"suspended_by\": {\n      \"oneOf\": [\n        {\n          \"$ref\": \"#/definitions/pickingone-user\"\n        },\n        {\n          \"type\": \"null\"\n        }\n      ]\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct PickingoneInstallation {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub suspended_by: ::std::option::Option<PickingoneUser>,
}
impl ::std::default::Default for PickingoneInstallation {
    fn default() -> Self {
        Self {
            suspended_by: ::std::default::Default::default(),
        }
    }
}
impl PickingoneInstallation {
    pub fn builder() -> builder::PickingoneInstallation {
        ::std::default::Default::default()
    }
}
#[doc = "`PickingoneSuspendedBy`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"type\": \"object\",\n      \"properties\": {\n        \"email\": {\n          \"type\": [\n            \"string\",\n            \"null\"\n          ]\n        }\n      }\n    },\n    {\n      \"$ref\": \"#/definitions/pickingone-user\"\n    },\n    {\n      \"not\": {\n        \"type\": \"null\"\n      }\n    }\n  ]\n}\n ```\n </details>"]
pub use self::PickingoneUser as PickingoneSuspendedBy;
#[doc = "`PickingoneUser`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"properties\": {\n    \"email\": {\n      \"type\": [\n        \"string\",\n        \"null\"\n      ]\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct PickingoneUser {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub email: ::std::option::Option<::std::string::String>,
}
impl ::std::default::Default for PickingoneUser {
    fn default() -> Self {
        Self {
            email: ::std::default::Default::default(),
        }
    }
}
impl PickingoneUser {
    pub fn builder() -> builder::PickingoneUser {
        ::std::default::Default::default()
    }
}
#[doc = "`TrimFat`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"not\": {\n    \"anyOf\": [\n      {\n        \"required\": [\n          \"b\"\n        ]\n      },\n      {\n        \"required\": [\n          \"c\"\n        ]\n      }\n    ]\n  },\n  \"required\": [\n    \"a\"\n  ],\n  \"properties\": {\n    \"a\": {},\n    \"b\": {},\n    \"c\": {}\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TrimFat {
    pub a: ::serde_json::Value,
}
impl TrimFat {
    pub fn builder() -> builder::TrimFat {
        ::std::default::Default::default()
    }
}
#[doc = "`TriplePattern`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"type\": \"string\",\n      \"format\": \"custom-id\",\n      \"pattern\": \"^[a-z].+$\"\n    },\n    {\n      \"type\": \"string\",\n      \"pattern\": \"^.{4,8}$\"\n    },\n    {\n      \"type\": \"string\",\n      \"pattern\": \".+[a-z]$\"\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TriplePattern(::std::string::String);
impl ::std::ops::Deref for TriplePattern {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TriplePattern> for ::std::string::String {
    fn from(value: TriplePattern) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TriplePattern {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| {
                ::regress::Regex::new("(?=^[a-z].+$)(?=^.{4,8}$)(?=.+[a-z]$)").unwrap()
            });
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"(?=^[a-z].+$)(?=^.{4,8}$)(?=.+[a-z]$)\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TriplePattern {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TriplePattern {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TriplePattern {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TriplePattern {
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
#[doc = "`UnchangedByMerge`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"type\": \"object\",\n      \"required\": [\n        \"tag\"\n      ],\n      \"properties\": {\n        \"tag\": {\n          \"enum\": [\n            \"something\"\n          ]\n        }\n      }\n    },\n    {\n      \"not\": {\n        \"type\": \"object\",\n        \"required\": [\n          \"tag\"\n        ],\n        \"properties\": {\n          \"tag\": {\n            \"enum\": [\n              \"something_else\"\n            ]\n          }\n        }\n      }\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct UnchangedByMerge {
    pub tag: UnchangedByMergeTag,
}
impl UnchangedByMerge {
    pub fn builder() -> builder::UnchangedByMerge {
        ::std::default::Default::default()
    }
}
#[doc = "`UnchangedByMergeTag`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"enum\": [\n    \"something\"\n  ]\n}\n ```\n </details>"]
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
pub enum UnchangedByMergeTag {
    #[serde(rename = "something")]
    Something,
}
impl ::std::fmt::Display for UnchangedByMergeTag {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Something => f.write_str("something"),
        }
    }
}
impl ::std::str::FromStr for UnchangedByMergeTag {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "something" => Ok(Self::Something),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for UnchangedByMergeTag {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for UnchangedByMergeTag {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for UnchangedByMergeTag {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Unresolvable`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"oneOf\": [\n    {\n      \"type\": \"object\",\n      \"required\": [\n        \"x\"\n      ],\n      \"properties\": {\n        \"x\": {\n          \"enum\": [\n            \"a\"\n          ]\n        }\n      }\n    },\n    {\n      \"type\": \"object\",\n      \"required\": [\n        \"x\"\n      ],\n      \"properties\": {\n        \"x\": {\n          \"enum\": [\n            \"b\"\n          ]\n        }\n      }\n    }\n  ],\n  \"required\": [\n    \"x\"\n  ],\n  \"properties\": {\n    \"x\": {\n      \"enum\": [\n        \"c\"\n      ]\n    }\n  },\n  \"$comment\": \"subschemas all end up unresolvable\"\n}\n ```\n </details>"]
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
pub enum Unresolvable {}
#[doc = "`Unsatisfiable1`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"type\": \"string\",\n      \"enum\": [\n        \"foo\"\n      ]\n    },\n    {\n      \"type\": \"object\",\n      \"properties\": {\n        \"bar\": {}\n      }\n    }\n  ]\n}\n ```\n </details>"]
pub use self::Unresolvable as Unsatisfiable1;
#[doc = "`Unsatisfiable2`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"type\": \"object\",\n      \"required\": [\n        \"action\"\n      ],\n      \"properties\": {\n        \"action\": {\n          \"type\": \"string\",\n          \"enum\": [\n            \"foo\"\n          ]\n        }\n      },\n      \"additionalProperties\": false\n    },\n    {\n      \"type\": \"object\",\n      \"properties\": {\n        \"action\": {\n          \"type\": \"string\",\n          \"enum\": [\n            \"bar\"\n          ]\n        }\n      },\n      \"additionalProperties\": false\n    }\n  ],\n  \"$comment\": \"can't be satisfied because required properties conflict in their enum values\"\n}\n ```\n </details>"]
pub use self::Unresolvable as Unsatisfiable2;
#[doc = "`Unsatisfiable3`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"allOf\": [\n    {\n      \"$ref\": \"#/definitions/unsatisfiable-3-a\"\n    },\n    {\n      \"type\": \"object\",\n      \"required\": [\n        \"action\"\n      ],\n      \"properties\": {\n        \"action\": {\n          \"$ref\": \"#/definitions/unsatisfiable-3-b\"\n        }\n      }\n    }\n  ],\n  \"$comment\": \"tests a complex merge that can't be satisfied; it's basically the same as unsatisfiable-2, but is broken into multiple pieces\"\n}\n ```\n </details>"]
pub use self::Unresolvable as Unsatisfiable3;
#[doc = "`Unsatisfiable3A`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"properties\": {\n    \"action\": {\n      \"allOf\": [\n        {\n          \"$ref\": \"#/definitions/unsatisfiable-3-c\"\n        }\n      ]\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Unsatisfiable3A {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub action: ::std::option::Option<Unsatisfiable3C>,
}
impl ::std::default::Default for Unsatisfiable3A {
    fn default() -> Self {
        Self {
            action: ::std::default::Default::default(),
        }
    }
}
impl Unsatisfiable3A {
    pub fn builder() -> builder::Unsatisfiable3A {
        ::std::default::Default::default()
    }
}
#[doc = "`Unsatisfiable3B`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\",\n  \"enum\": [\n    \"bar\"\n  ]\n}\n ```\n </details>"]
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
pub enum Unsatisfiable3B {
    #[serde(rename = "bar")]
    Bar,
}
impl ::std::fmt::Display for Unsatisfiable3B {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Bar => f.write_str("bar"),
        }
    }
}
impl ::std::str::FromStr for Unsatisfiable3B {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "bar" => Ok(Self::Bar),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Unsatisfiable3B {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Unsatisfiable3B {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Unsatisfiable3B {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Unsatisfiable3C`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\",\n  \"enum\": [\n    \"foo\"\n  ]\n}\n ```\n </details>"]
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
pub enum Unsatisfiable3C {
    #[serde(rename = "foo")]
    Foo,
}
impl ::std::fmt::Display for Unsatisfiable3C {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Foo => f.write_str("foo"),
        }
    }
}
impl ::std::str::FromStr for Unsatisfiable3C {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "foo" => Ok(Self::Foo),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Unsatisfiable3C {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Unsatisfiable3C {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Unsatisfiable3C {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`WeirdEnum`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"oneOf\": [\n    {\n      \"not\": {\n        \"anyOf\": [\n          {\n            \"required\": [\n              \"patterns\"\n            ]\n          },\n          {\n            \"required\": [\n              \"pattern-either\"\n            ]\n          },\n          {\n            \"required\": [\n              \"pattern-regex\"\n            ]\n          }\n        ]\n      },\n      \"required\": [\n        \"pattern\"\n      ]\n    },\n    {\n      \"not\": {\n        \"anyOf\": [\n          {\n            \"required\": [\n              \"pattern\"\n            ]\n          },\n          {\n            \"required\": [\n              \"pattern-either\"\n            ]\n          },\n          {\n            \"required\": [\n              \"pattern-regex\"\n            ]\n          }\n        ]\n      },\n      \"required\": [\n        \"patterns\"\n      ]\n    },\n    {\n      \"not\": {\n        \"anyOf\": [\n          {\n            \"required\": [\n              \"pattern\"\n            ]\n          },\n          {\n            \"required\": [\n              \"patterns\"\n            ]\n          },\n          {\n            \"required\": [\n              \"pattern-regex\"\n            ]\n          }\n        ]\n      },\n      \"required\": [\n        \"pattern-either\"\n      ]\n    },\n    {\n      \"not\": {\n        \"anyOf\": [\n          {\n            \"required\": [\n              \"pattern\"\n            ]\n          },\n          {\n            \"required\": [\n              \"patterns\"\n            ]\n          },\n          {\n            \"required\": [\n              \"pattern-either\"\n            ]\n          }\n        ]\n      },\n      \"required\": [\n        \"pattern-regex\"\n      ]\n    }\n  ],\n  \"properties\": {\n    \"pattern\": {\n      \"type\": \"string\"\n    },\n    \"pattern-either\": {\n      \"type\": \"string\"\n    },\n    \"pattern-regex\": {\n      \"type\": \"string\"\n    },\n    \"patterns\": {\n      \"type\": \"string\"\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum WeirdEnum {
    Variant0 {
        pattern: ::std::string::String,
    },
    Variant1 {
        patterns: ::std::string::String,
    },
    Variant2 {
        #[serde(rename = "pattern-either")]
        pattern_either: ::std::string::String,
    },
    Variant3 {
        #[serde(rename = "pattern-regex")]
        pattern_regex: ::std::string::String,
    },
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct BarProp {
        bar: ::std::result::Result<
            ::std::option::Option<::serde_json::Value>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for BarProp {
        fn default() -> Self {
            Self {
                bar: Ok(::std::default::Default::default()),
            }
        }
    }
    impl BarProp {
        pub fn bar<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::serde_json::Value>>,
            T::Error: ::std::fmt::Display,
        {
            self.bar = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for bar: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<BarProp> for super::BarProp {
        type Error = super::error::ConversionError;
        fn try_from(value: BarProp) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { bar: value.bar? })
        }
    }
    impl ::std::convert::From<super::BarProp> for BarProp {
        fn from(value: super::BarProp) -> Self {
            Self { bar: Ok(value.bar) }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ButNotThat {
        this: ::std::result::Result<
            ::std::option::Option<::serde_json::Value>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for ButNotThat {
        fn default() -> Self {
            Self {
                this: Ok(::std::default::Default::default()),
            }
        }
    }
    impl ButNotThat {
        pub fn this<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::serde_json::Value>>,
            T::Error: ::std::fmt::Display,
        {
            self.this = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for this: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ButNotThat> for super::ButNotThat {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ButNotThat,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { this: value.this? })
        }
    }
    impl ::std::convert::From<super::ButNotThat> for ButNotThat {
        fn from(value: super::ButNotThat) -> Self {
            Self {
                this: Ok(value.this),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CommentedTypeMerged {
        x: ::std::result::Result<::std::option::Option<::serde_json::Value>, ::std::string::String>,
        y: ::std::result::Result<::std::option::Option<::serde_json::Value>, ::std::string::String>,
    }
    impl ::std::default::Default for CommentedTypeMerged {
        fn default() -> Self {
            Self {
                x: Ok(::std::default::Default::default()),
                y: Ok(::std::default::Default::default()),
            }
        }
    }
    impl CommentedTypeMerged {
        pub fn x<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::serde_json::Value>>,
            T::Error: ::std::fmt::Display,
        {
            self.x = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for x: {e}"));
            self
        }
        pub fn y<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::serde_json::Value>>,
            T::Error: ::std::fmt::Display,
        {
            self.y = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for y: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CommentedTypeMerged> for super::CommentedTypeMerged {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CommentedTypeMerged,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                x: value.x?,
                y: value.y?,
            })
        }
    }
    impl ::std::convert::From<super::CommentedTypeMerged> for CommentedTypeMerged {
        fn from(value: super::CommentedTypeMerged) -> Self {
            Self {
                x: Ok(value.x),
                y: Ok(value.y),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct JsonResponseBase {
        result: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for JsonResponseBase {
        fn default() -> Self {
            Self {
                result: Ok(::std::default::Default::default()),
            }
        }
    }
    impl JsonResponseBase {
        pub fn result<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.result = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for result: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<JsonResponseBase> for super::JsonResponseBase {
        type Error = super::error::ConversionError;
        fn try_from(
            value: JsonResponseBase,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                result: value.result?,
            })
        }
    }
    impl ::std::convert::From<super::JsonResponseBase> for JsonResponseBase {
        fn from(value: super::JsonResponseBase) -> Self {
            Self {
                result: Ok(value.result),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct JsonSuccess {
        msg: ::std::result::Result<::std::string::String, ::std::string::String>,
        result: ::std::result::Result<super::JsonSuccessResult, ::std::string::String>,
    }
    impl ::std::default::Default for JsonSuccess {
        fn default() -> Self {
            Self {
                msg: Err("no value supplied for msg".to_string()),
                result: Err("no value supplied for result".to_string()),
            }
        }
    }
    impl JsonSuccess {
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
        pub fn result<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JsonSuccessResult>,
            T::Error: ::std::fmt::Display,
        {
            self.result = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for result: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<JsonSuccess> for super::JsonSuccess {
        type Error = super::error::ConversionError;
        fn try_from(
            value: JsonSuccess,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                msg: value.msg?,
                result: value.result?,
            })
        }
    }
    impl ::std::convert::From<super::JsonSuccess> for JsonSuccess {
        fn from(value: super::JsonSuccess) -> Self {
            Self {
                msg: Ok(value.msg),
                result: Ok(value.result),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct JsonSuccessBase {
        msg: ::std::result::Result<::std::string::String, ::std::string::String>,
        result: ::std::result::Result<super::JsonSuccessBaseResult, ::std::string::String>,
    }
    impl ::std::default::Default for JsonSuccessBase {
        fn default() -> Self {
            Self {
                msg: Err("no value supplied for msg".to_string()),
                result: Err("no value supplied for result".to_string()),
            }
        }
    }
    impl JsonSuccessBase {
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
        pub fn result<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JsonSuccessBaseResult>,
            T::Error: ::std::fmt::Display,
        {
            self.result = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for result: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<JsonSuccessBase> for super::JsonSuccessBase {
        type Error = super::error::ConversionError;
        fn try_from(
            value: JsonSuccessBase,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                msg: value.msg?,
                result: value.result?,
            })
        }
    }
    impl ::std::convert::From<super::JsonSuccessBase> for JsonSuccessBase {
        fn from(value: super::JsonSuccessBase) -> Self {
            Self {
                msg: Ok(value.msg),
                result: Ok(value.result),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MergeEmpty {}
    impl ::std::default::Default for MergeEmpty {
        fn default() -> Self {
            Self {}
        }
    }
    impl MergeEmpty {}
    impl ::std::convert::TryFrom<MergeEmpty> for super::MergeEmpty {
        type Error = super::error::ConversionError;
        fn try_from(
            _value: MergeEmpty,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {})
        }
    }
    impl ::std::convert::From<super::MergeEmpty> for MergeEmpty {
        fn from(_value: super::MergeEmpty) -> Self {
            Self {}
        }
    }
    #[derive(Clone, Debug)]
    pub struct OrderDependentMerge {
        bar: ::std::result::Result<
            ::std::option::Option<::serde_json::Value>,
            ::std::string::String,
        >,
        baz: ::std::result::Result<bool, ::std::string::String>,
    }
    impl ::std::default::Default for OrderDependentMerge {
        fn default() -> Self {
            Self {
                bar: Ok(::std::default::Default::default()),
                baz: Err("no value supplied for baz".to_string()),
            }
        }
    }
    impl OrderDependentMerge {
        pub fn bar<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::serde_json::Value>>,
            T::Error: ::std::fmt::Display,
        {
            self.bar = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for bar: {e}"));
            self
        }
        pub fn baz<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.baz = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for baz: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<OrderDependentMerge> for super::OrderDependentMerge {
        type Error = super::error::ConversionError;
        fn try_from(
            value: OrderDependentMerge,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                bar: value.bar?,
                baz: value.baz?,
            })
        }
    }
    impl ::std::convert::From<super::OrderDependentMerge> for OrderDependentMerge {
        fn from(value: super::OrderDependentMerge) -> Self {
            Self {
                bar: Ok(value.bar),
                baz: Ok(value.baz),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Pickingone {
        suspended_by: ::std::result::Result<super::PickingoneSuspendedBy, ::std::string::String>,
    }
    impl ::std::default::Default for Pickingone {
        fn default() -> Self {
            Self {
                suspended_by: Err("no value supplied for suspended_by".to_string()),
            }
        }
    }
    impl Pickingone {
        pub fn suspended_by<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PickingoneSuspendedBy>,
            T::Error: ::std::fmt::Display,
        {
            self.suspended_by = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for suspended_by: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Pickingone> for super::Pickingone {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Pickingone,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                suspended_by: value.suspended_by?,
            })
        }
    }
    impl ::std::convert::From<super::Pickingone> for Pickingone {
        fn from(value: super::Pickingone) -> Self {
            Self {
                suspended_by: Ok(value.suspended_by),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct PickingoneInstallation {
        suspended_by: ::std::result::Result<
            ::std::option::Option<super::PickingoneUser>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for PickingoneInstallation {
        fn default() -> Self {
            Self {
                suspended_by: Ok(::std::default::Default::default()),
            }
        }
    }
    impl PickingoneInstallation {
        pub fn suspended_by<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::PickingoneUser>>,
            T::Error: ::std::fmt::Display,
        {
            self.suspended_by = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for suspended_by: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<PickingoneInstallation> for super::PickingoneInstallation {
        type Error = super::error::ConversionError;
        fn try_from(
            value: PickingoneInstallation,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                suspended_by: value.suspended_by?,
            })
        }
    }
    impl ::std::convert::From<super::PickingoneInstallation> for PickingoneInstallation {
        fn from(value: super::PickingoneInstallation) -> Self {
            Self {
                suspended_by: Ok(value.suspended_by),
            }
        }
    }
    pub use self::PickingoneUser as PickingoneSuspendedBy;
    #[derive(Clone, Debug)]
    pub struct PickingoneUser {
        email: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for PickingoneUser {
        fn default() -> Self {
            Self {
                email: Ok(::std::default::Default::default()),
            }
        }
    }
    impl PickingoneUser {
        pub fn email<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.email = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for email: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<PickingoneUser> for super::PickingoneUser {
        type Error = super::error::ConversionError;
        fn try_from(
            value: PickingoneUser,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                email: value.email?,
            })
        }
    }
    impl ::std::convert::From<super::PickingoneUser> for PickingoneUser {
        fn from(value: super::PickingoneUser) -> Self {
            Self {
                email: Ok(value.email),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TrimFat {
        a: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for TrimFat {
        fn default() -> Self {
            Self {
                a: Err("no value supplied for a".to_string()),
            }
        }
    }
    impl TrimFat {
        pub fn a<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.a = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for a: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TrimFat> for super::TrimFat {
        type Error = super::error::ConversionError;
        fn try_from(value: TrimFat) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { a: value.a? })
        }
    }
    impl ::std::convert::From<super::TrimFat> for TrimFat {
        fn from(value: super::TrimFat) -> Self {
            Self { a: Ok(value.a) }
        }
    }
    #[derive(Clone, Debug)]
    pub struct UnchangedByMerge {
        tag: ::std::result::Result<super::UnchangedByMergeTag, ::std::string::String>,
    }
    impl ::std::default::Default for UnchangedByMerge {
        fn default() -> Self {
            Self {
                tag: Err("no value supplied for tag".to_string()),
            }
        }
    }
    impl UnchangedByMerge {
        pub fn tag<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::UnchangedByMergeTag>,
            T::Error: ::std::fmt::Display,
        {
            self.tag = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tag: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<UnchangedByMerge> for super::UnchangedByMerge {
        type Error = super::error::ConversionError;
        fn try_from(
            value: UnchangedByMerge,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { tag: value.tag? })
        }
    }
    impl ::std::convert::From<super::UnchangedByMerge> for UnchangedByMerge {
        fn from(value: super::UnchangedByMerge) -> Self {
            Self { tag: Ok(value.tag) }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Unsatisfiable3A {
        action: ::std::result::Result<
            ::std::option::Option<super::Unsatisfiable3C>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Unsatisfiable3A {
        fn default() -> Self {
            Self {
                action: Ok(::std::default::Default::default()),
            }
        }
    }
    impl Unsatisfiable3A {
        pub fn action<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Unsatisfiable3C>>,
            T::Error: ::std::fmt::Display,
        {
            self.action = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for action: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Unsatisfiable3A> for super::Unsatisfiable3A {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Unsatisfiable3A,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                action: value.action?,
            })
        }
    }
    impl ::std::convert::From<super::Unsatisfiable3A> for Unsatisfiable3A {
        fn from(value: super::Unsatisfiable3A) -> Self {
            Self {
                action: Ok(value.action),
            }
        }
    }
}
fn main() {}
