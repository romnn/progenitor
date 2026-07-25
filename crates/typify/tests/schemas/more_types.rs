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
#[doc = "`ObjectWithNoExtra`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"required\": [\n    \"foo\"\n  ],\n  \"properties\": {\n    \"foo\": {\n      \"type\": \"string\"\n    }\n  },\n  \"additionalProperties\": false\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ObjectWithNoExtra {
    pub foo: ::std::string::String,
}
impl ObjectWithNoExtra {
    pub fn builder() -> builder::ObjectWithNoExtra {
        ::std::default::Default::default()
    }
}
#[doc = "`ObjectWithOkExtra`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"required\": [\n    \"foo\"\n  ],\n  \"properties\": {\n    \"foo\": {\n      \"type\": \"string\"\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct ObjectWithOkExtra {
    pub foo: ::std::string::String,
}
impl ObjectWithOkExtra {
    pub fn builder() -> builder::ObjectWithOkExtra {
        ::std::default::Default::default()
    }
}
#[doc = "`ObjectWithStringExtra`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"required\": [\n    \"foo\"\n  ],\n  \"properties\": {\n    \"foo\": {\n      \"type\": \"string\"\n    }\n  },\n  \"additionalProperties\": {\n    \"type\": \"string\"\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct ObjectWithStringExtra {
    pub foo: ::std::string::String,
    #[serde(flatten)]
    pub extra: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
}
impl ObjectWithStringExtra {
    pub fn builder() -> builder::ObjectWithStringExtra {
        ::std::default::Default::default()
    }
}
#[doc = "`ObjectWithWhichExtra`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"required\": [\n    \"foo\"\n  ],\n  \"properties\": {\n    \"foo\": {\n      \"type\": \"string\"\n    }\n  },\n  \"additionalProperties\": {}\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct ObjectWithWhichExtra {
    pub foo: ::std::string::String,
    #[serde(flatten)]
    pub extra: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
}
impl ObjectWithWhichExtra {
    pub fn builder() -> builder::ObjectWithWhichExtra {
        ::std::default::Default::default()
    }
}
#[doc = "`ObjectWithYesExtra`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"required\": [\n    \"foo\"\n  ],\n  \"properties\": {\n    \"foo\": {\n      \"type\": \"string\"\n    }\n  },\n  \"additionalProperties\": true\n}\n ```\n </details>"]
pub use self::ObjectWithOkExtra as ObjectWithYesExtra;
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct ObjectWithNoExtra {
        foo: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for ObjectWithNoExtra {
        fn default() -> Self {
            Self {
                foo: Err("no value supplied for foo".to_string()),
            }
        }
    }
    impl ObjectWithNoExtra {
        pub fn foo<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.foo = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for foo: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ObjectWithNoExtra> for super::ObjectWithNoExtra {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ObjectWithNoExtra,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { foo: value.foo? })
        }
    }
    impl ::std::convert::From<super::ObjectWithNoExtra> for ObjectWithNoExtra {
        fn from(value: super::ObjectWithNoExtra) -> Self {
            Self { foo: Ok(value.foo) }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ObjectWithOkExtra {
        foo: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for ObjectWithOkExtra {
        fn default() -> Self {
            Self {
                foo: Err("no value supplied for foo".to_string()),
            }
        }
    }
    impl ObjectWithOkExtra {
        pub fn foo<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.foo = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for foo: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ObjectWithOkExtra> for super::ObjectWithOkExtra {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ObjectWithOkExtra,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { foo: value.foo? })
        }
    }
    impl ::std::convert::From<super::ObjectWithOkExtra> for ObjectWithOkExtra {
        fn from(value: super::ObjectWithOkExtra) -> Self {
            Self { foo: Ok(value.foo) }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ObjectWithStringExtra {
        foo: ::std::result::Result<::std::string::String, ::std::string::String>,
        extra: ::std::result::Result<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for ObjectWithStringExtra {
        fn default() -> Self {
            Self {
                foo: Err("no value supplied for foo".to_string()),
                extra: Err("no value supplied for extra".to_string()),
            }
        }
    }
    impl ObjectWithStringExtra {
        pub fn foo<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.foo = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for foo: {e}"));
            self
        }
        pub fn extra<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::collections::HashMap<::std::string::String, ::std::string::String>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.extra = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for extra: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ObjectWithStringExtra> for super::ObjectWithStringExtra {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ObjectWithStringExtra,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                foo: value.foo?,
                extra: value.extra?,
            })
        }
    }
    impl ::std::convert::From<super::ObjectWithStringExtra> for ObjectWithStringExtra {
        fn from(value: super::ObjectWithStringExtra) -> Self {
            Self {
                foo: Ok(value.foo),
                extra: Ok(value.extra),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ObjectWithWhichExtra {
        foo: ::std::result::Result<::std::string::String, ::std::string::String>,
        extra: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for ObjectWithWhichExtra {
        fn default() -> Self {
            Self {
                foo: Err("no value supplied for foo".to_string()),
                extra: Err("no value supplied for extra".to_string()),
            }
        }
    }
    impl ObjectWithWhichExtra {
        pub fn foo<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.foo = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for foo: {e}"));
            self
        }
        pub fn extra<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.extra = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for extra: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ObjectWithWhichExtra> for super::ObjectWithWhichExtra {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ObjectWithWhichExtra,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                foo: value.foo?,
                extra: value.extra?,
            })
        }
    }
    impl ::std::convert::From<super::ObjectWithWhichExtra> for ObjectWithWhichExtra {
        fn from(value: super::ObjectWithWhichExtra) -> Self {
            Self {
                foo: Ok(value.foo),
                extra: Ok(value.extra),
            }
        }
    }
    pub use self::ObjectWithOkExtra as ObjectWithYesExtra;
}
fn main() {}
