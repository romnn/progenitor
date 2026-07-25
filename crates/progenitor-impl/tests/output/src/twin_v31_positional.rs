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

    ///`CreateThing`
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
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct CreateThing {
        pub name: ::std::string::String,
    }

    ///`Dog`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "petType"
    ///  ],
    ///  "properties": {
    ///    "bark": {
    ///      "type": "boolean"
    ///    },
    ///    "petType": {
    ///      "type": "string",
    ///      "enum": [
    ///        "Dog"
    ///      ]
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Dog {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub bark: ::std::option::Option<bool>,
        #[serde(rename = "petType")]
        pub pet_type: DogPetType,
    }

    ///`DogPetType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "string",
    ///  "enum": [
    ///    "Dog"
    ///  ]
    /// }
    /// ```
    /// </details>
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
    pub enum DogPetType {
        Dog,
    }

    impl ::std::fmt::Display for DogPetType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Dog => f.write_str("Dog"),
            }
        }
    }

    impl ::std::str::FromStr for DogPetType {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "Dog" => Ok(Self::Dog),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for DogPetType {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for DogPetType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for DogPetType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`Error`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "message"
    ///  ],
    ///  "properties": {
    ///    "message": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Error {
        pub message: ::std::string::String,
    }

    ///`Label`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "text"
    ///  ],
    ///  "properties": {
    ///    "text": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Label {
        pub text: ::std::string::String,
    }

    ///`Pet`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "oneOf": [
    ///    {
    ///      "$ref": "#/components/schemas/Dog"
    ///    }
    ///  ]
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    #[serde(tag = "petType")]
    pub enum Pet {
        Dog {
            #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
            bark: ::std::option::Option<bool>,
        },
    }

    ///`Thing`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "kind"
    ///  ],
    ///  "properties": {
    ///    "count": {
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "either": {
    ///      "oneOf": [
    ///        {
    ///          "type": "null"
    ///        },
    ///        {
    ///          "oneOf": [
    ///            {
    ///              "$ref": "#/components/schemas/Label"
    ///            },
    ///            {
    ///              "$ref": "#/components/schemas/Error"
    ///            }
    ///          ]
    ///        }
    ///      ]
    ///    },
    ///    "id": {
    ///      "type": "string",
    ///      "format": "uuid"
    ///    },
    ///    "kind": {
    ///      "type": "string",
    ///      "enum": [
    ///        "fixed"
    ///      ]
    ///    },
    ///    "labels": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Label"
    ///      }
    ///    },
    ///    "note": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "weight": {
    ///      "type": "number",
    ///      "exclusiveMinimum": 0.0
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Thing {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub count: ::std::option::Option<u64>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub either: ::std::option::Option<ThingEither>,
        pub id: ::uuid::Uuid,
        pub kind: ThingKind,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        pub labels: ::std::vec::Vec<Label>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub note: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub weight: ::std::option::Option<f64>,
    }

    ///`ThingEither`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "oneOf": [
    ///    {
    ///      "$ref": "#/components/schemas/Label"
    ///    },
    ///    {
    ///      "$ref": "#/components/schemas/Error"
    ///    }
    ///  ]
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    #[serde(untagged)]
    pub enum ThingEither {
        Label(Label),
        Error(Error),
    }

    impl ::std::convert::From<Label> for ThingEither {
        fn from(value: Label) -> Self {
            Self::Label(value)
        }
    }

    impl ::std::convert::From<Error> for ThingEither {
        fn from(value: Error) -> Self {
            Self::Error(value)
        }
    }

    ///`ThingKind`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "string",
    ///  "enum": [
    ///    "fixed"
    ///  ]
    /// }
    /// ```
    /// </details>
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
    pub enum ThingKind {
        #[serde(rename = "fixed")]
        Fixed,
    }

    impl ::std::fmt::Display for ThingKind {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Fixed => f.write_str("fixed"),
            }
        }
    }

    impl ::std::str::FromStr for ThingKind {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "fixed" => Ok(Self::Fixed),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for ThingKind {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for ThingKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for ThingKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
}

#[derive(Clone, Debug)]
///Client for Twin
///
///Twin-spec equivalence fixture
///
///Version: 1.0.0
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
}

impl ClientInfo<()> for Client {
    fn api_version() -> &'static str {
        "1.0.0"
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
    ///Sends a `GET` request to `/things/{id}`
    pub async fn get_thing<'a>(
        &'a self,
        id: &'a ::uuid::Uuid,
        verbose: ::std::option::Option<bool>,
    ) -> Result<ResponseValue<types::Thing>, Error<types::Error>> {
        let url = format!("{}/things/{}", self.baseurl, encode_path(&id.to_string()),);
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
            .query(&progenitor_client::QueryParam::new("verbose", &verbose))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "get_thing",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::ErrorResponse(
                ResponseValue::from_response(response).await?,
            )),
        }
    }

    ///Sends a `POST` request to `/things`
    pub async fn create_thing<'a>(
        &'a self,
        body: &'a types::CreateThing,
    ) -> Result<ResponseValue<types::Thing>, Error<()>> {
        let url = format!("{}/things", self.baseurl,);
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
            operation_id: "create_thing",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            201u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `GET` request to `/pets`
    pub async fn get_pet<'a>(&'a self) -> Result<ResponseValue<types::Pet>, Error<()>> {
        let url = format!("{}/pets", self.baseurl,);
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
            operation_id: "get_pet",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `POST` request to `/upload`
    pub async fn upload_blob<'a, B: Into<reqwest::Body>>(
        &'a self,
        body: B,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/upload", self.baseurl,);
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
                ::reqwest::header::CONTENT_TYPE,
                ::reqwest::header::HeaderValue::from_static("application/octet-stream"),
            )
            .body(body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "upload_blob",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            204u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `PUT` request to `/note`
    pub async fn set_note<'a>(
        &'a self,
        body: ::std::string::String,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/note", self.baseurl,);
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
            operation_id: "set_note",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            204u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
}

/// Items consumers will typically use such as the Client.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
}
