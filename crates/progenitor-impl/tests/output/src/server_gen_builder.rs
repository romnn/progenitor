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

    impl Error {
        pub fn builder() -> builder::Error {
            ::std::default::Default::default()
        }
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

    impl Item {
        pub fn builder() -> builder::Item {
            ::std::default::Default::default()
        }
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
    /// Types for composing complex structures.
    pub mod builder {
        #[derive(Clone, Debug)]
        pub struct Error {
            code: ::std::result::Result<i32, ::std::string::String>,
            message: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for Error {
            fn default() -> Self {
                Self {
                    code: Err("no value supplied for code".to_string()),
                    message: Err("no value supplied for message".to_string()),
                }
            }
        }

        impl Error {
            pub fn code<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i32>,
                T::Error: ::std::fmt::Display,
            {
                self.code = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for code: {e}"));
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
        }

        impl ::std::convert::TryFrom<Error> for super::Error {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Error,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    code: value.code?,
                    message: value.message?,
                })
            }
        }

        impl ::std::convert::From<super::Error> for Error {
            fn from(value: super::Error) -> Self {
                Self {
                    code: Ok(value.code),
                    message: Ok(value.message),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Item {
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for Item {
            fn default() -> Self {
                Self {
                    id: Err("no value supplied for id".to_string()),
                    name: Err("no value supplied for name".to_string()),
                }
            }
        }

        impl Item {
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {e}"));
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

        impl ::std::convert::TryFrom<Item> for super::Item {
            type Error = super::error::ConversionError;
            fn try_from(value: Item) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    id: value.id?,
                    name: value.name?,
                })
            }
        }

        impl ::std::convert::From<super::Item> for Item {
            fn from(value: super::Item) -> Self {
                Self {
                    id: Ok(value.id),
                    name: Ok(value.name),
                }
            }
        }
    }
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
impl Client {
    ///List items, optionally limited
    ///
    ///Sends a `GET` request to `/items`
    ///
    ///```ignore
    /// let response = client.list_items()
    ///    .limit(limit)
    ///    .x_trace(x_trace)
    ///    .send()
    ///    .await;
    /// ```
    pub fn list_items(&self) -> builder::ListItems<'_> {
        builder::ListItems::new(self)
    }

    ///Create an item (JSON body, 201 with no content)
    ///
    ///Sends a `POST` request to `/items`
    ///
    ///```ignore
    /// let response = client.create_item()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn create_item(&self) -> builder::CreateItem<'_> {
        builder::CreateItem::new(self)
    }

    ///Fetch one item by id
    ///
    ///Sends a `GET` request to `/items/{itemId}`
    ///
    ///```ignore
    /// let response = client.get_item()
    ///    .item_id(item_id)
    ///    .send()
    ///    .await;
    /// ```
    pub fn get_item(&self) -> builder::GetItem<'_> {
        builder::GetItem::new(self)
    }

    ///Update an item (path + query + JSON body)
    ///
    ///Sends a `PUT` request to `/items/{itemId}`
    ///
    ///```ignore
    /// let response = client.update_item()
    ///    .item_id(item_id)
    ///    .dry_run(dry_run)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn update_item(&self) -> builder::UpdateItem<'_> {
        builder::UpdateItem::new(self)
    }

    ///Params named like the framework bindings, plus a required array query
    ///
    ///Sends a `GET` request to `/collide/{inner}`
    ///
    ///```ignore
    /// let response = client.collide()
    ///    .inner(inner)
    ///    .query(query)
    ///    .tags(tags)
    ///    .meta(meta)
    ///    .send()
    ///    .await;
    /// ```
    pub fn collide(&self) -> builder::Collide<'_> {
        builder::Collide::new(self)
    }

    ///Raw octet-stream response body
    ///
    ///Sends a `GET` request to `/blob`
    ///
    ///```ignore
    /// let response = client.download_blob()
    ///    .send()
    ///    .await;
    /// ```
    pub fn download_blob(&self) -> builder::DownloadBlob<'_> {
        builder::DownloadBlob::new(self)
    }

    ///Mixed JSON/raw success and error responses
    ///
    ///Sends a `GET` request to `/multi/{mode}`
    ///
    ///```ignore
    /// let response = client.multi_kind()
    ///    .mode(mode)
    ///    .send()
    ///    .await;
    /// ```
    pub fn multi_kind(&self) -> builder::MultiKind<'_> {
        builder::MultiKind::new(self)
    }

    ///Upload typed multipart fields
    ///
    ///Sends a `POST` request to `/upload/{file}`
    ///
    ///```ignore
    /// let response = client.upload_item()
    ///    .file(file)
    ///    .multipart_file_part(multipart_file_part)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn upload_item(&self) -> builder::UploadItem<'_> {
        builder::UploadItem::new(self)
    }

    ///Keep schema-less multipart as a raw body
    ///
    ///Sends a `POST` request to `/upload-raw`
    ///
    ///```ignore
    /// let response = client.upload_raw()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    pub fn upload_raw(&self) -> builder::UploadRaw<'_> {
        builder::UploadRaw::new(self)
    }

    ///An operation whose name collides with the service rejection hook
    ///
    ///Sends a `POST` request to `/render-rejection`
    ///
    ///```ignore
    /// let response = client.render_rejection()
    ///    .send()
    ///    .await;
    /// ```
    pub fn render_rejection(&self) -> builder::RenderRejection<'_> {
        builder::RenderRejection::new(self)
    }

    ///deepObject query params are emitted as a 501 server stub
    ///
    ///Sends a `GET` request to `/search`
    ///
    ///```ignore
    /// let response = client.deep_search()
    ///    .filter(filter)
    ///    .send()
    ///    .await;
    /// ```
    pub fn deep_search(&self) -> builder::DeepSearch<'_> {
        builder::DeepSearch::new(self)
    }

    ///An upgrade endpoint — generates a 501 route stub, no trait method
    ///
    ///Sends a `GET` request to `/upgrade`
    ///
    ///```ignore
    /// let response = client.do_upgrade()
    ///    .send()
    ///    .await;
    /// ```
    pub fn do_upgrade(&self) -> builder::DoUpgrade<'_> {
        builder::DoUpgrade::new(self)
    }

    ///Path axum cannot route: literal suffix after a parameter
    ///
    ///Sends a `GET` request to `/files/{fileId}.json`
    ///
    ///```ignore
    /// let response = client.suffix_after_param()
    ///    .file_id(file_id)
    ///    .send()
    ///    .await;
    /// ```
    pub fn suffix_after_param(&self) -> builder::SuffixAfterParam<'_> {
        builder::SuffixAfterParam::new(self)
    }

    ///Shape shared with /shape/{second} via a different method
    ///
    ///Sends a `GET` request to `/shape/{first}`
    ///
    ///```ignore
    /// let response = client.shape_by_first()
    ///    .first(first)
    ///    .send()
    ///    .await;
    /// ```
    pub fn shape_by_first(&self) -> builder::ShapeByFirst<'_> {
        builder::ShapeByFirst::new(self)
    }

    ///Same shape as /shape/{first}, different method: merges
    ///
    ///Sends a `POST` request to `/shape/{second}`
    ///
    ///```ignore
    /// let response = client.shape_by_second()
    ///    .second(second)
    ///    .send()
    ///    .await;
    /// ```
    pub fn shape_by_second(&self) -> builder::ShapeBySecond<'_> {
        builder::ShapeBySecond::new(self)
    }

    ///Same shape AND method as /shape/{first}: dropped
    ///
    ///Sends a `GET` request to `/shape/{third}`
    ///
    ///```ignore
    /// let response = client.shape_by_third()
    ///    .third(third)
    ///    .send()
    ///    .await;
    /// ```
    pub fn shape_by_third(&self) -> builder::ShapeByThird<'_> {
        builder::ShapeByThird::new(self)
    }

    ///Template names the same path parameter twice
    ///
    ///Sends a `GET` request to `/repeat/{outer}/mid/{inner}/tail/{outer}`
    ///
    ///```ignore
    /// let response = client.repeated_path_param()
    ///    .inner(inner)
    ///    .outer(outer)
    ///    .send()
    ///    .await;
    /// ```
    pub fn repeated_path_param(&self) -> builder::RepeatedPathParam<'_> {
        builder::RepeatedPathParam::new(self)
    }
}

/// Types for composing operation parameters.
#[allow(clippy::all)]
pub mod builder {
    use super::types;
    #[allow(unused_imports)]
    use super::{
        encode_path, multipart_file_part, ByteStream, ClientHooks, ClientInfo, Error, FilePart,
        OperationInfo, RequestBuilderExt, ResponseValue,
    };
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

    ///Builder for [`Client::list_items`]
    ///
    ///[`Client::list_items`]: super::Client::list_items
    #[derive(Debug, Clone)]
    pub struct ListItems<'a> {
        client: &'a super::Client,
        limit: ::std::result::Result<::std::option::Option<i32>, ::std::string::String>,
        x_trace: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }

    impl<'a> ListItems<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                limit: Ok(None),
                x_trace: Ok(None),
            }
        }

        pub fn limit<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i32>,
        {
            self.limit = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `i32` for limit failed".to_string());
            self
        }

        pub fn x_trace<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.x_trace = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for x_trace failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/items`
        pub async fn send(self) -> Result<ResponseValue<types::ItemList>, Error<types::Error>> {
            let Self {
                client,
                limit,
                x_trace,
            } = self;
            let limit = limit.map_err(Error::InvalidRequest)?;
            let x_trace = x_trace.map_err(Error::InvalidRequest)?;
            let url = format!("{}/items", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            if let Some(value) = x_trace {
                header_map.append("x-trace", value.to_string().try_into()?);
            }
            #[allow(unused_mut)]
            let mut request = client
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
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
            }
        }
    }

    ///Builder for [`Client::create_item`]
    ///
    ///[`Client::create_item`]: super::Client::create_item
    #[derive(Debug, Clone)]
    pub struct CreateItem<'a> {
        client: &'a super::Client,
        body: ::std::result::Result<types::builder::Item, ::std::string::String>,
    }

    impl<'a> CreateItem<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::Item>,
            <V as std::convert::TryInto<types::Item>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `Item` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::Item) -> types::builder::Item,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/items`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<types::Error>> {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::Item::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/items", client.baseurl,);
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
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "create_item",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                201u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
            }
        }
    }

    ///Builder for [`Client::get_item`]
    ///
    ///[`Client::get_item`]: super::Client::get_item
    #[derive(Debug, Clone)]
    pub struct GetItem<'a> {
        client: &'a super::Client,
        item_id: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> GetItem<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                item_id: Err("item_id was not initialized".to_string()),
            }
        }

        pub fn item_id<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.item_id = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for item_id failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/items/{itemId}`
        pub async fn send(self) -> Result<ResponseValue<types::Item>, Error<types::Error>> {
            let Self { client, item_id } = self;
            let item_id = item_id.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/items/{}",
                client.baseurl,
                encode_path(&item_id.to_string()),
            );
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
                operation_id: "get_item",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
            }
        }
    }

    ///Builder for [`Client::update_item`]
    ///
    ///[`Client::update_item`]: super::Client::update_item
    #[derive(Debug, Clone)]
    pub struct UpdateItem<'a> {
        client: &'a super::Client,
        item_id: ::std::result::Result<::std::string::String, ::std::string::String>,
        dry_run: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        body: ::std::result::Result<types::builder::Item, ::std::string::String>,
    }

    impl<'a> UpdateItem<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                item_id: Err("item_id was not initialized".to_string()),
                dry_run: Ok(None),
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn item_id<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.item_id = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for item_id failed".to_string()
            });
            self
        }

        pub fn dry_run<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.dry_run = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for dry_run failed".to_string());
            self
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::Item>,
            <V as std::convert::TryInto<types::Item>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `Item` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::Item) -> types::builder::Item,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `PUT` request to `/items/{itemId}`
        pub async fn send(self) -> Result<ResponseValue<types::Item>, Error<types::Error>> {
            let Self {
                client,
                item_id,
                dry_run,
                body,
            } = self;
            let item_id = item_id.map_err(Error::InvalidRequest)?;
            let dry_run = dry_run.map_err(Error::InvalidRequest)?;
            let body = body
                .and_then(|v| types::Item::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/items/{}",
                client.baseurl,
                encode_path(&item_id.to_string()),
            );
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
                .query(&progenitor_client::QueryParam::new("dryRun", &dry_run))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "update_item",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
            }
        }
    }

    ///Builder for [`Client::collide`]
    ///
    ///[`Client::collide`]: super::Client::collide
    #[derive(Debug, Clone)]
    pub struct Collide<'a> {
        client: &'a super::Client,
        inner: ::std::result::Result<::std::string::String, ::std::string::String>,
        query: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        tags: ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        meta: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }

    impl<'a> Collide<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                inner: Err("inner was not initialized".to_string()),
                query: Ok(None),
                tags: Err("tags was not initialized".to_string()),
                meta: Ok(None),
            }
        }

        pub fn inner<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.inner = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for inner failed".to_string()
            });
            self
        }

        pub fn query<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.query = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for query failed".to_string()
            });
            self
        }

        pub fn tags<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
        {
            self.tags = value.try_into().map_err(|_| {
                "conversion to `:: std :: vec :: Vec < :: std :: string :: String >` for tags \
                 failed"
                    .to_string()
            });
            self
        }

        pub fn meta<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.meta = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for meta failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/collide/{inner}`
        pub async fn send(self) -> Result<ResponseValue<types::Message>, Error<()>> {
            let Self {
                client,
                inner,
                query,
                tags,
                meta,
            } = self;
            let inner = inner.map_err(Error::InvalidRequest)?;
            let query = query.map_err(Error::InvalidRequest)?;
            let tags = tags.map_err(Error::InvalidRequest)?;
            let meta = meta.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/collide/{}",
                client.baseurl,
                encode_path(&inner.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(2usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            if let Some(value) = meta {
                header_map.append("meta", value.to_string().try_into()?);
            }
            #[allow(unused_mut)]
            let mut request = client
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
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::download_blob`]
    ///
    ///[`Client::download_blob`]: super::Client::download_blob
    #[derive(Debug, Clone)]
    pub struct DownloadBlob<'a> {
        client: &'a super::Client,
    }

    impl<'a> DownloadBlob<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/blob`
        pub async fn send(self) -> Result<ResponseValue<ByteStream>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/blob", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.get(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "download_blob",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200u16 => Ok(ResponseValue::stream(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::multi_kind`]
    ///
    ///[`Client::multi_kind`]: super::Client::multi_kind
    #[derive(Debug, Clone)]
    pub struct MultiKind<'a> {
        client: &'a super::Client,
        mode: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> MultiKind<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                mode: Err("mode was not initialized".to_string()),
            }
        }

        pub fn mode<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.mode = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for mode failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/multi/{mode}`
        pub async fn send(self) -> Result<ResponseValue<MultiKindResponse>, Error<MultiKindError>> {
            let Self { client, mode } = self;
            let mode = mode.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/multi/{}",
                client.baseurl,
                encode_path(&mode.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.get(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "multi_kind",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                200u16 => Ok(ResponseValue::<types::Message>::from_response(response)
                    .await?
                    .map(|inner| MultiKindResponse::Status200(inner))),
                206u16 => Ok(ResponseValue::stream(response)
                    .map(|inner| MultiKindResponse::Status206(inner))),
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
    }

    ///Builder for [`Client::upload_item`]
    ///
    ///[`Client::upload_item`]: super::Client::upload_item
    #[derive(Debug, Clone)]
    pub struct UploadItem<'a> {
        client: &'a super::Client,
        file: ::std::result::Result<::std::string::String, ::std::string::String>,
        multipart_file_part: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        body: ::std::result::Result<UploadItemMultipartBody, ::std::string::String>,
    }

    impl<'a> UploadItem<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                file: Err("file was not initialized".to_string()),
                multipart_file_part: Ok(None),
                body: Err("body was not initialized".to_string()),
            }
        }

        pub fn file<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.file = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for file failed".to_string()
            });
            self
        }

        pub fn multipart_file_part<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.multipart_file_part = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for multipart_file_part failed"
                    .to_string()
            });
            self
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<UploadItemMultipartBody>,
        {
            self.body = value
                .try_into()
                .map_err(|_| "conversion to `UploadItemMultipartBody` for body failed".to_string());
            self
        }

        ///Sends a `POST` request to `/upload/{file}`
        pub async fn send(self) -> Result<ResponseValue<types::Message>, Error<()>> {
            let Self {
                client,
                file,
                multipart_file_part,
                body,
            } = self;
            let file = file.map_err(Error::InvalidRequest)?;
            let multipart_file_part = multipart_file_part.map_err(Error::InvalidRequest)?;
            let body = body.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/upload/{}",
                client.baseurl,
                encode_path(&file.to_string()),
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
                .multipart({
                    let mut __progenitor_multipart_form = ::reqwest::multipart::Form::new();
                    for value in body.attachments {
                        __progenitor_multipart_form = __progenitor_multipart_form
                            .part("attachments", self::multipart_file_part(value)?);
                    }
                    __progenitor_multipart_form = __progenitor_multipart_form
                        .part("file", self::multipart_file_part(body.file)?);
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
            client
                .__progenitor_response(request, &info, &[(201u16, 201u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::upload_raw`]
    ///
    ///[`Client::upload_raw`]: super::Client::upload_raw
    #[derive(Debug)]
    pub struct UploadRaw<'a> {
        client: &'a super::Client,
        body: ::std::result::Result<reqwest::Body, ::std::string::String>,
    }

    impl<'a> UploadRaw<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Err("body was not initialized".to_string()),
            }
        }

        pub fn body<B>(mut self, value: B) -> Self
        where
            B: std::convert::TryInto<reqwest::Body>,
        {
            self.body = value
                .try_into()
                .map_err(|_| "conversion to `reqwest::Body` for body failed".to_string());
            self
        }

        ///Sends a `POST` request to `/upload-raw`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client, body } = self;
            let body = body.map_err(Error::InvalidRequest)?;
            let url = format!("{}/upload-raw", client.baseurl,);
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
                    ::reqwest::header::CONTENT_TYPE,
                    ::reqwest::header::HeaderValue::from_static("multipart/form-data"),
                )
                .body(body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "upload_raw",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                204u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::render_rejection`]
    ///
    ///[`Client::render_rejection`]: super::Client::render_rejection
    #[derive(Debug, Clone)]
    pub struct RenderRejection<'a> {
        client: &'a super::Client,
    }

    impl<'a> RenderRejection<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `POST` request to `/render-rejection`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/render-rejection", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.post(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "render_rejection",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                204u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::deep_search`]
    ///
    ///[`Client::deep_search`]: super::Client::deep_search
    #[derive(Debug, Clone)]
    pub struct DeepSearch<'a> {
        client: &'a super::Client,
        filter: ::std::result::Result<
            ::std::option::Option<
                ::std::collections::HashMap<::std::string::String, ::std::string::String>,
            >,
            ::std::string::String,
        >,
    }

    impl<'a> DeepSearch<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                filter: Ok(None),
            }
        }

        pub fn filter<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<
                ::std::collections::HashMap<::std::string::String, ::std::string::String>,
            >,
        {
            self.filter = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: collections :: HashMap < :: std :: string :: String , :: \
                 std :: string :: String >` for filter failed"
                    .to_string()
            });
            self
        }

        ///Sends a `GET` request to `/search`
        pub async fn send(self) -> Result<ResponseValue<types::ItemList>, Error<()>> {
            let Self { client, filter } = self;
            let filter = filter.map_err(Error::InvalidRequest)?;
            let url = format!("{}/search", client.baseurl,);
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
                .query(&progenitor_client::DeepObjectQuery::new("filter", &filter))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "deep_search",
            };
            client
                .__progenitor_response(request, &info, &[(200u16, 200u16)], &[])
                .await
        }
    }

    ///Builder for [`Client::do_upgrade`]
    ///
    ///[`Client::do_upgrade`]: super::Client::do_upgrade
    #[derive(Debug, Clone)]
    pub struct DoUpgrade<'a> {
        client: &'a super::Client,
    }

    impl<'a> DoUpgrade<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/upgrade`
        pub async fn send(self) -> Result<ResponseValue<reqwest::Upgraded>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/upgrade", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.get(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "do_upgrade",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                101u16 => ResponseValue::upgrade(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::suffix_after_param`]
    ///
    ///[`Client::suffix_after_param`]: super::Client::suffix_after_param
    #[derive(Debug, Clone)]
    pub struct SuffixAfterParam<'a> {
        client: &'a super::Client,
        file_id: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> SuffixAfterParam<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                file_id: Err("file_id was not initialized".to_string()),
            }
        }

        pub fn file_id<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.file_id = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for file_id failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/files/{fileId}.json`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client, file_id } = self;
            let file_id = file_id.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/files/{}.json",
                client.baseurl,
                encode_path(&file_id.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.get(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "suffix_after_param",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                204u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::shape_by_first`]
    ///
    ///[`Client::shape_by_first`]: super::Client::shape_by_first
    #[derive(Debug, Clone)]
    pub struct ShapeByFirst<'a> {
        client: &'a super::Client,
        first: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> ShapeByFirst<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                first: Err("first was not initialized".to_string()),
            }
        }

        pub fn first<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.first = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for first failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/shape/{first}`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client, first } = self;
            let first = first.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/shape/{}",
                client.baseurl,
                encode_path(&first.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.get(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "shape_by_first",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                204u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::shape_by_second`]
    ///
    ///[`Client::shape_by_second`]: super::Client::shape_by_second
    #[derive(Debug, Clone)]
    pub struct ShapeBySecond<'a> {
        client: &'a super::Client,
        second: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> ShapeBySecond<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                second: Err("second was not initialized".to_string()),
            }
        }

        pub fn second<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.second = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for second failed".to_string()
            });
            self
        }

        ///Sends a `POST` request to `/shape/{second}`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client, second } = self;
            let second = second.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/shape/{}",
                client.baseurl,
                encode_path(&second.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.post(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "shape_by_second",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                204u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::shape_by_third`]
    ///
    ///[`Client::shape_by_third`]: super::Client::shape_by_third
    #[derive(Debug, Clone)]
    pub struct ShapeByThird<'a> {
        client: &'a super::Client,
        third: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> ShapeByThird<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                third: Err("third was not initialized".to_string()),
            }
        }

        pub fn third<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.third = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for third failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/shape/{third}`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self { client, third } = self;
            let third = third.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/shape/{}",
                client.baseurl,
                encode_path(&third.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.get(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "shape_by_third",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                204u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::repeated_path_param`]
    ///
    ///[`Client::repeated_path_param`]: super::Client::repeated_path_param
    #[derive(Debug, Clone)]
    pub struct RepeatedPathParam<'a> {
        client: &'a super::Client,
        inner: ::std::result::Result<::std::string::String, ::std::string::String>,
        outer: ::std::result::Result<::std::string::String, ::std::string::String>,
    }

    impl<'a> RepeatedPathParam<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                inner: Err("inner was not initialized".to_string()),
                outer: Err("outer was not initialized".to_string()),
            }
        }

        pub fn inner<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.inner = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for inner failed".to_string()
            });
            self
        }

        pub fn outer<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.outer = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for outer failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/repeat/{outer}/mid/{inner}/tail/{outer}`
        pub async fn send(self) -> Result<ResponseValue<()>, Error<()>> {
            let Self {
                client,
                inner,
                outer,
            } = self;
            let inner = inner.map_err(Error::InvalidRequest)?;
            let outer = outer.map_err(Error::InvalidRequest)?;
            let url = format!(
                "{}/repeat/{}/mid/{}/tail/{}",
                client.baseurl,
                encode_path(&outer.to_string()),
                encode_path(&inner.to_string()),
                encode_path(&outer.to_string()),
            );
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.get(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "repeated_path_param",
            };
            let response = client.__progenitor_dispatch(request, &info).await?;
            match response.status().as_u16() {
                204u16 => Ok(ResponseValue::empty(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }
}

/// Items consumers will typically use such as the Client.
pub mod prelude {
    pub use self::super::Client;
}
