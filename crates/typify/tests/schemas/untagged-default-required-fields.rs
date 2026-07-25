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
#[doc = "`Breakpoints`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"required\": [\n    \"small\"\n  ],\n  \"properties\": {\n    \"small\": {\n      \"type\": \"integer\"\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Breakpoints {
    pub small: i64,
}
impl Breakpoints {
    pub fn builder() -> builder::Breakpoints {
        ::std::default::Default::default()
    }
}
#[doc = "`Layout`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"required\": [\n    \"breakpoints\",\n    \"version\"\n  ],\n  \"properties\": {\n    \"breakpoints\": {\n      \"$ref\": \"#/definitions/Breakpoints\"\n    },\n    \"version\": {\n      \"type\": \"integer\",\n      \"enum\": [\n        1\n      ]\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Layout {
    pub breakpoints: Breakpoints,
    pub version: LayoutVersion,
}
impl Layout {
    pub fn builder() -> builder::Layout {
        ::std::default::Default::default()
    }
}
#[doc = "`LayoutVersion`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"integer\",\n  \"enum\": [\n    1\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct LayoutVersion(i64);
impl ::std::ops::Deref for LayoutVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<LayoutVersion> for i64 {
    fn from(value: LayoutVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for LayoutVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for LayoutVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`Missing`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"object\",\n  \"required\": [\n    \"kind\"\n  ],\n  \"properties\": {\n    \"kind\": {\n      \"type\": \"string\",\n      \"enum\": [\n        \"missing\"\n      ]\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Missing {
    pub kind: MissingKind,
}
impl Missing {
    pub fn builder() -> builder::Missing {
        ::std::default::Default::default()
    }
}
#[doc = "`MissingKind`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"type\": \"string\",\n  \"enum\": [\n    \"missing\"\n  ]\n}\n ```\n </details>"]
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
pub enum MissingKind {
    #[serde(rename = "missing")]
    Missing,
}
impl ::std::fmt::Display for MissingKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Missing => f.write_str("missing"),
        }
    }
}
impl ::std::str::FromStr for MissingKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "missing" => Ok(Self::Missing),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MissingKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MissingKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MissingKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Update`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"title\": \"Update\",\n  \"type\": \"object\",\n  \"properties\": {\n    \"layout\": {\n      \"default\": {\n        \"kind\": \"missing\"\n      },\n      \"anyOf\": [\n        {\n          \"$ref\": \"#/definitions/Layout\"\n        },\n        {\n          \"$ref\": \"#/definitions/Missing\"\n        }\n      ]\n    }\n  }\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Update {
    #[serde(default = "defaults::update_layout")]
    pub layout: UpdateLayout,
}
impl ::std::default::Default for Update {
    fn default() -> Self {
        Self {
            layout: defaults::update_layout(),
        }
    }
}
impl Update {
    pub fn builder() -> builder::Update {
        ::std::default::Default::default()
    }
}
#[doc = "`UpdateLayout`"]
#[doc = "\n <details><summary>JSON schema</summary>\n\n ```json\n{\n  \"default\": {\n    \"kind\": \"missing\"\n  },\n  \"anyOf\": [\n    {\n      \"$ref\": \"#/definitions/Layout\"\n    },\n    {\n      \"$ref\": \"#/definitions/Missing\"\n    }\n  ]\n}\n ```\n </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum UpdateLayout {
    Layout(Layout),
    Missing(Missing),
}
impl ::std::default::Default for UpdateLayout {
    fn default() -> Self {
        UpdateLayout::Missing(Missing {
            kind: MissingKind::Missing,
        })
    }
}
impl ::std::convert::From<Layout> for UpdateLayout {
    fn from(value: Layout) -> Self {
        Self::Layout(value)
    }
}
impl ::std::convert::From<Missing> for UpdateLayout {
    fn from(value: Missing) -> Self {
        Self::Missing(value)
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct Breakpoints {
        small: ::std::result::Result<i64, ::std::string::String>,
    }
    impl ::std::default::Default for Breakpoints {
        fn default() -> Self {
            Self {
                small: Err("no value supplied for small".to_string()),
            }
        }
    }
    impl Breakpoints {
        pub fn small<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.small = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for small: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Breakpoints> for super::Breakpoints {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Breakpoints,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                small: value.small?,
            })
        }
    }
    impl ::std::convert::From<super::Breakpoints> for Breakpoints {
        fn from(value: super::Breakpoints) -> Self {
            Self {
                small: Ok(value.small),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Layout {
        breakpoints: ::std::result::Result<super::Breakpoints, ::std::string::String>,
        version: ::std::result::Result<super::LayoutVersion, ::std::string::String>,
    }
    impl ::std::default::Default for Layout {
        fn default() -> Self {
            Self {
                breakpoints: Err("no value supplied for breakpoints".to_string()),
                version: Err("no value supplied for version".to_string()),
            }
        }
    }
    impl Layout {
        pub fn breakpoints<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Breakpoints>,
            T::Error: ::std::fmt::Display,
        {
            self.breakpoints = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for breakpoints: {e}"));
            self
        }
        pub fn version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::LayoutVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for version: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Layout> for super::Layout {
        type Error = super::error::ConversionError;
        fn try_from(value: Layout) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                breakpoints: value.breakpoints?,
                version: value.version?,
            })
        }
    }
    impl ::std::convert::From<super::Layout> for Layout {
        fn from(value: super::Layout) -> Self {
            Self {
                breakpoints: Ok(value.breakpoints),
                version: Ok(value.version),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Missing {
        kind: ::std::result::Result<super::MissingKind, ::std::string::String>,
    }
    impl ::std::default::Default for Missing {
        fn default() -> Self {
            Self {
                kind: Err("no value supplied for kind".to_string()),
            }
        }
    }
    impl Missing {
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MissingKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Missing> for super::Missing {
        type Error = super::error::ConversionError;
        fn try_from(value: Missing) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { kind: value.kind? })
        }
    }
    impl ::std::convert::From<super::Missing> for Missing {
        fn from(value: super::Missing) -> Self {
            Self {
                kind: Ok(value.kind),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Update {
        layout: ::std::result::Result<super::UpdateLayout, ::std::string::String>,
    }
    impl ::std::default::Default for Update {
        fn default() -> Self {
            Self {
                layout: Ok(super::defaults::update_layout()),
            }
        }
    }
    impl Update {
        pub fn layout<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::UpdateLayout>,
            T::Error: ::std::fmt::Display,
        {
            self.layout = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for layout: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Update> for super::Update {
        type Error = super::error::ConversionError;
        fn try_from(value: Update) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                layout: value.layout?,
            })
        }
    }
    impl ::std::convert::From<super::Update> for Update {
        fn from(value: super::Update) -> Self {
            Self {
                layout: Ok(value.layout),
            }
        }
    }
}
#[doc = r" Generation of default values for serde."]
pub mod defaults {
    pub(super) fn update_layout() -> super::UpdateLayout {
        super::UpdateLayout::Missing(super::Missing {
            kind: super::MissingKind::Missing,
        })
    }
}
fn main() {}
