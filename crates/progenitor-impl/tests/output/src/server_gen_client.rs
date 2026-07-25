#[allow(unused_imports)]
use progenitor_client::{
    encode_path, multipart_file_part, ClientHooks, OperationInfo, RequestBuilderExt,
};
#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, ClientInfo, Error, FilePart, ResponseValue};
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

    ///`DoesNotExist`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// true
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    #[serde(transparent)]
    pub struct DoesNotExist(pub ::serde_json::Value);
    impl ::std::ops::Deref for DoesNotExist {
        type Target = ::serde_json::Value;
        fn deref(&self) -> &::serde_json::Value {
            &self.0
        }
    }

    impl ::std::convert::From<DoesNotExist> for ::serde_json::Value {
        fn from(value: DoesNotExist) -> Self {
            value.0
        }
    }

    impl ::std::convert::From<::serde_json::Value> for DoesNotExist {
        fn from(value: ::serde_json::Value) -> Self {
            Self(value)
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
    ///    "code",
    ///    "message"
    ///  ],
    ///  "properties": {
    ///    "code": {
    ///      "type": "integer",
    ///      "format": "int32"
    ///    },
    ///    "message": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Error {
        pub code: i32,
        pub message: ::std::string::String,
    }

    ///`Item`
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
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Item {
        pub id: ::std::string::String,
        pub name: ::std::string::String,
    }

    ///`ItemList`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "type": "array",
    ///  "items": {
    ///    "$ref": "#/components/schemas/Item"
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    #[serde(transparent)]
    pub struct ItemList(pub ::std::vec::Vec<Item>);
    impl ::std::ops::Deref for ItemList {
        type Target = ::std::vec::Vec<Item>;
        fn deref(&self) -> &::std::vec::Vec<Item> {
            &self.0
        }
    }

    impl ::std::convert::From<ItemList> for ::std::vec::Vec<Item> {
        fn from(value: ItemList) -> Self {
            value.0
        }
    }

    impl ::std::convert::From<::std::vec::Vec<Item>> for ItemList {
        fn from(value: ::std::vec::Vec<Item>) -> Self {
            Self(value)
        }
    }

    ///`Message`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// true
    /// ```
    /// </details>
    pub use self::DoesNotExist as Message;
}

#[derive(Clone, Debug)]
///Client for ServerGen
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
pub enum MultiKindResponse {
    Status200(types::Message),
    Status206(ByteStream),
}

pub enum MultiKindError {
    Status401(types::Error),
    StatusRange4xx(types::Error),
    Default(ByteStream),
}

///Typed multipart request body for the `upload_item` operation.
#[derive(Debug, Clone)]
pub struct UploadItemMultipartBody {
    ///The `attachments` multipart field.
    pub attachments: ::std::vec::Vec<FilePart>,
    ///The primary upload.
    pub file: FilePart,
    ///The `legacy` multipart field.
    pub legacy: ::std::option::Option<::std::string::String>,
    ///The `metadata` multipart field.
    pub metadata: ::std::option::Option<::std::string::String>,
    ///A note stored with the upload.
    pub note: ::std::option::Option<::std::string::String>,
    ///The `rating` multipart field.
    pub rating: i32,
    ///A field named like the generated request URL local.
    pub url: ::std::option::Option<::std::string::String>,
}

#[allow(clippy::all)]
impl Client {
    ///List items, optionally limited
    ///
    ///Sends a `GET` request to `/items`
    pub async fn list_items<'a>(
        &'a self,
        limit: ::std::option::Option<i32>,
        x_trace: ::std::option::Option<&'a str>,
    ) -> Result<ResponseValue<types::ItemList>, Error<types::Error>> {
        let url = format!("{}/items", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        if let Some(value) = x_trace {
            header_map.append("x-trace", value.to_string().try_into()?);
        }

        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("limit", &limit))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "list_items",
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

    ///Create an item (JSON body, 201 with no content)
    ///
    ///Sends a `POST` request to `/items`
    pub async fn create_item<'a>(
        &'a self,
        body: &'a types::Item,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!("{}/items", self.baseurl,);
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
            operation_id: "create_item",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            201u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::ErrorResponse(
                ResponseValue::from_response(response).await?,
            )),
        }
    }

    ///Fetch one item by id
    ///
    ///Sends a `GET` request to `/items/{itemId}`
    pub async fn get_item<'a>(
        &'a self,
        item_id: &'a str,
    ) -> Result<ResponseValue<types::Item>, Error<types::Error>> {
        let url = format!(
            "{}/items/{}",
            self.baseurl,
            encode_path(&item_id.to_string()),
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
            operation_id: "get_item",
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

    ///Update an item (path + query + JSON body)
    ///
    ///Sends a `PUT` request to `/items/{itemId}`
    pub async fn update_item<'a>(
        &'a self,
        item_id: &'a str,
        dry_run: ::std::option::Option<bool>,
        body: &'a types::Item,
    ) -> Result<ResponseValue<types::Item>, Error<types::Error>> {
        let url = format!(
            "{}/items/{}",
            self.baseurl,
            encode_path(&item_id.to_string()),
        );
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
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("dryRun", &dry_run))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "update_item",
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

    ///Params named like the framework bindings, plus a required array query
    ///
    ///Sends a `GET` request to `/collide/{inner}`
    pub async fn collide<'a>(
        &'a self,
        inner: &'a str,
        query: ::std::option::Option<&'a str>,
        tags: &'a ::std::vec::Vec<::std::string::String>,
        meta: ::std::option::Option<&'a str>,
    ) -> Result<ResponseValue<types::Message>, Error<()>> {
        let url = format!(
            "{}/collide/{}",
            self.baseurl,
            encode_path(&inner.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        if let Some(value) = meta {
            header_map.append("meta", value.to_string().try_into()?);
        }

        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("query", &query))
            .query(&progenitor_client::QueryParam::new("tags", &tags))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "collide",
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

    ///Raw octet-stream response body
    ///
    ///Sends a `GET` request to `/blob`
    pub async fn download_blob<'a>(&'a self) -> Result<ResponseValue<ByteStream>, Error<()>> {
        let url = format!("{}/blob", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "download_blob",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::stream(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Mixed JSON/raw success and error responses
    ///
    ///Sends a `GET` request to `/multi/{mode}`
    pub async fn multi_kind<'a>(
        &'a self,
        mode: &'a str,
    ) -> Result<ResponseValue<MultiKindResponse>, Error<MultiKindError>> {
        let url = format!("{}/multi/{}", self.baseurl, encode_path(&mode.to_string()),);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "multi_kind",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::<types::Message>::from_response(response)
                .await?
                .map(|inner| MultiKindResponse::Status200(inner))),
            206u16 => Ok(
                ResponseValue::stream(response).map(|inner| MultiKindResponse::Status206(inner))
            ),
            401u16 => Err(Error::ErrorResponse(
                ResponseValue::<types::Error>::from_response(response)
                    .await?
                    .map(|inner| MultiKindError::Status401(inner)),
            )),
            400u16..=499u16 => Err(Error::ErrorResponse(
                ResponseValue::<types::Error>::from_response(response)
                    .await?
                    .map(|inner| MultiKindError::StatusRange4xx(inner)),
            )),
            _ => Err(Error::ErrorResponse(
                ResponseValue::stream(response).map(|inner| MultiKindError::Default(inner)),
            )),
        }
    }

    ///Upload typed multipart fields
    ///
    ///Sends a `POST` request to `/upload/{file}`
    pub async fn upload_item<'a>(
        &'a self,
        file: &'a str,
        multipart_file_part: ::std::option::Option<&'a str>,
        body: UploadItemMultipartBody,
    ) -> Result<ResponseValue<types::Message>, Error<()>> {
        let url = format!("{}/upload/{}", self.baseurl, encode_path(&file.to_string()),);
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
            .multipart({
                let mut __progenitor_multipart_form = ::reqwest::multipart::Form::new();
                for value in body.attachments {
                    __progenitor_multipart_form = __progenitor_multipart_form
                        .part("attachments", self::multipart_file_part(value)?);
                }
                __progenitor_multipart_form =
                    __progenitor_multipart_form.part("file", self::multipart_file_part(body.file)?);
                if let Some(value) = body.legacy {
                    __progenitor_multipart_form =
                        __progenitor_multipart_form.text("legacy", value.to_string());
                }
                if let Some(value) = body.metadata {
                    __progenitor_multipart_form =
                        __progenitor_multipart_form.text("metadata", value.to_string());
                }
                if let Some(value) = body.note {
                    __progenitor_multipart_form =
                        __progenitor_multipart_form.text("note", value.to_string());
                }
                __progenitor_multipart_form =
                    __progenitor_multipart_form.text("rating", body.rating.to_string());
                if let Some(value) = body.url {
                    __progenitor_multipart_form =
                        __progenitor_multipart_form.text("url", value.to_string());
                }
                __progenitor_multipart_form
            })
            .query(&progenitor_client::QueryParam::new(
                "multipart_file_part",
                &multipart_file_part,
            ))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "upload_item",
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

    ///Keep schema-less multipart as a raw body
    ///
    ///Sends a `POST` request to `/upload-raw`
    pub async fn upload_raw<'a, B: Into<reqwest::Body>>(
        &'a self,
        body: B,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/upload-raw", self.baseurl,);
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
                ::reqwest::header::HeaderValue::from_static("multipart/form-data"),
            )
            .body(body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "upload_raw",
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

    ///An operation whose name collides with the service rejection hook
    ///
    ///Sends a `POST` request to `/render-rejection`
    pub async fn render_rejection<'a>(&'a self) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/render-rejection", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.post(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "render_rejection",
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

    ///deepObject query params are emitted as a 501 server stub
    ///
    ///Sends a `GET` request to `/search`
    pub async fn deep_search<'a>(
        &'a self,
        filter: ::std::option::Option<
            &'a ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
    ) -> Result<ResponseValue<types::ItemList>, Error<()>> {
        let url = format!("{}/search", self.baseurl,);
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
            .query(&progenitor_client::DeepObjectQuery::new("filter", &filter))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "deep_search",
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

    ///An upgrade endpoint — generates a 501 route stub, no trait method
    ///
    ///Sends a `GET` request to `/upgrade`
    pub async fn do_upgrade<'a>(&'a self) -> Result<ResponseValue<reqwest::Upgraded>, Error<()>> {
        let url = format!("{}/upgrade", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "do_upgrade",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            101u16 => ResponseValue::upgrade(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Path axum cannot route: literal suffix after a parameter
    ///
    ///Sends a `GET` request to `/files/{fileId}.json`
    pub async fn suffix_after_param<'a>(
        &'a self,
        file_id: &'a str,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!(
            "{}/files/{}.json",
            self.baseurl,
            encode_path(&file_id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "suffix_after_param",
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

    ///Shape shared with /shape/{second} via a different method
    ///
    ///Sends a `GET` request to `/shape/{first}`
    pub async fn shape_by_first<'a>(
        &'a self,
        first: &'a str,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/shape/{}", self.baseurl, encode_path(&first.to_string()),);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "shape_by_first",
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

    ///Same shape as /shape/{first}, different method: merges
    ///
    ///Sends a `POST` request to `/shape/{second}`
    pub async fn shape_by_second<'a>(
        &'a self,
        second: &'a str,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!(
            "{}/shape/{}",
            self.baseurl,
            encode_path(&second.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.post(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "shape_by_second",
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

    ///Same shape AND method as /shape/{first}: dropped
    ///
    ///Sends a `GET` request to `/shape/{third}`
    pub async fn shape_by_third<'a>(
        &'a self,
        third: &'a str,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/shape/{}", self.baseurl, encode_path(&third.to_string()),);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "shape_by_third",
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

    ///Template names the same path parameter twice
    ///
    ///Sends a `GET` request to `/repeat/{outer}/mid/{inner}/tail/{outer}`
    pub async fn repeated_path_param<'a>(
        &'a self,
        inner: &'a str,
        outer: &'a str,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!(
            "{}/repeat/{}/mid/{}/tail/{}",
            self.baseurl,
            encode_path(&outer.to_string()),
            encode_path(&inner.to_string()),
            encode_path(&outer.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map.append(
            ::reqwest::header::HeaderName::from_static("api-version"),
            ::reqwest::header::HeaderValue::from_static(Self::api_version()),
        );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "repeated_path_param",
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
