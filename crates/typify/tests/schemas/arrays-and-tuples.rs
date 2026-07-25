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
#[doc = "`ArraySansItems`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"array\",\n  \"minItems\": 1,\n  \"uniqueItems\": true\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct ArraySansItems(pub ::std::vec::Vec<::serde_json::Value>);
impl ::std::ops::Deref for ArraySansItems {
    type Target = ::std::vec::Vec<::serde_json::Value>;
    fn deref(&self) -> &::std::vec::Vec<::serde_json::Value> {
        &self.0
    }
}
impl ::std::convert::From<ArraySansItems> for ::std::vec::Vec<::serde_json::Value> {
    fn from(value: ArraySansItems) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::vec::Vec<::serde_json::Value>> for ArraySansItems {
    fn from(value: ::std::vec::Vec<::serde_json::Value>) -> Self {
        Self(value)
    }
}
#[doc = "`LessSimpleTwoTuple`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"array\",\n  \"items\": [\n    {\n      \"type\": \"string\"\n    },\n    {\n      \"type\": \"string\"\n    },\n    {\n      \"type\": \"string\"\n    }\n  ],\n  \"maxItems\": 2,\n  \"minItems\": 2\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct LessSimpleTwoTuple(pub (::std::string::String, ::std::string::String));
impl ::std::ops::Deref for LessSimpleTwoTuple {
    type Target = (::std::string::String, ::std::string::String);
    fn deref(&self) -> &(::std::string::String, ::std::string::String) {
        &self.0
    }
}
impl ::std::convert::From<LessSimpleTwoTuple> for (::std::string::String, ::std::string::String) {
    fn from(value: LessSimpleTwoTuple) -> Self {
        value.0
    }
}
impl ::std::convert::From<(::std::string::String, ::std::string::String)> for LessSimpleTwoTuple {
    fn from(value: (::std::string::String, ::std::string::String)) -> Self {
        Self(value)
    }
}
#[doc = "`SimpleTwoArray`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"array\",\n  \"items\": {\n    \"type\": \"string\"\n  },\n  \"maxItems\": 2,\n  \"minItems\": 2\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct SimpleTwoArray(pub [::std::string::String; 2usize]);
impl ::std::ops::Deref for SimpleTwoArray {
    type Target = [::std::string::String; 2usize];
    fn deref(&self) -> &[::std::string::String; 2usize] {
        &self.0
    }
}
impl ::std::convert::From<SimpleTwoArray> for [::std::string::String; 2usize] {
    fn from(value: SimpleTwoArray) -> Self {
        value.0
    }
}
impl ::std::convert::From<[::std::string::String; 2usize]> for SimpleTwoArray {
    fn from(value: [::std::string::String; 2usize]) -> Self {
        Self(value)
    }
}
#[doc = "`SimpleTwoTuple`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"array\",\n  \"items\": [\n    {\n      \"type\": \"string\"\n    },\n    {\n      \"type\": \"string\"\n    }\n  ],\n  \"maxItems\": 2,\n  \"minItems\": 2\n}\n ```\n </details>"]
pub use self::LessSimpleTwoTuple as SimpleTwoTuple;
#[doc = "`UnsimpleTwoTuple`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"array\",\n  \"items\": [\n    {\n      \"type\": \"string\"\n    }\n  ],\n  \"additionalItems\": {\n    \"type\": \"string\"\n  },\n  \"maxItems\": 2,\n  \"minItems\": 2\n}\n ```\n </details>"]
pub use self::LessSimpleTwoTuple as UnsimpleTwoTuple;
#[doc = "`YoloTwoArray`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"array\",\n  \"additionalItems\": {\n    \"type\": \"string\",\n    \"$comment\": \"ignored\"\n  },\n  \"maxItems\": 2,\n  \"minItems\": 2\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct YoloTwoArray(pub [::serde_json::Value; 2usize]);
impl ::std::ops::Deref for YoloTwoArray {
    type Target = [::serde_json::Value; 2usize];
    fn deref(&self) -> &[::serde_json::Value; 2usize] {
        &self.0
    }
}
impl ::std::convert::From<YoloTwoArray> for [::serde_json::Value; 2usize] {
    fn from(value: YoloTwoArray) -> Self {
        value.0
    }
}
impl ::std::convert::From<[::serde_json::Value; 2usize]> for YoloTwoArray {
    fn from(value: [::serde_json::Value; 2usize]) -> Self {
        Self(value)
    }
}
fn main() {}
