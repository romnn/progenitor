use crate::server_gen_client::types;
#[allow(unused_imports)]
use ::progenitor_server::codegen::*;
#[doc = "The declared responses represented by `ListItemsError`."]
#[derive(Debug, Clone)]
pub enum ListItemsErrorResponse {
    #[doc = "The default response."]
    Default {
        #[doc = r" The HTTP status selected for this response."]
        status: http::StatusCode,
        #[doc = r" The response body."]
        body: types::Error,
    },
}

impl ::std::convert::From<ListItemsErrorResponse>
    for ::progenitor_server::ServerError<ListItemsErrorResponse>
{
    fn from(body: ListItemsErrorResponse) -> Self {
        ::progenitor_server::ServerError::Api(body)
    }
}

#[doc = "Error type for the `list_items` operation."]
pub type ListItemsError = ::progenitor_server::ServerError<ListItemsErrorResponse>;
#[doc = "Bundled, typed request for the `list_items` operation."]
#[derive(Debug, Clone)]
pub struct ListItemsRequest {
    pub limit: Option<i32>,
    pub x_trace: Option<::std::string::String>,
}

#[derive(Debug, Clone, :: serde :: Deserialize)]
pub struct ListItemsQuery {
    #[serde(rename = "limit")]
    pub limit: Option<i32>,
}

#[doc = "The declared responses represented by `CreateItemError`."]
#[derive(Debug, Clone)]
pub enum CreateItemErrorResponse {
    #[doc = "The default response."]
    Default {
        #[doc = r" The HTTP status selected for this response."]
        status: http::StatusCode,
        #[doc = r" The response body."]
        body: types::Error,
    },
}

impl ::std::convert::From<CreateItemErrorResponse>
    for ::progenitor_server::ServerError<CreateItemErrorResponse>
{
    fn from(body: CreateItemErrorResponse) -> Self {
        ::progenitor_server::ServerError::Api(body)
    }
}

#[doc = "Error type for the `create_item` operation."]
pub type CreateItemError = ::progenitor_server::ServerError<CreateItemErrorResponse>;
#[doc = "Bundled, typed request for the `create_item` operation."]
#[derive(Debug, Clone)]
pub struct CreateItemRequest {
    pub body: types::Item,
}

#[doc = "The declared responses represented by `GetItemError`."]
#[derive(Debug, Clone)]
pub enum GetItemErrorResponse {
    #[doc = "The default response."]
    Default {
        #[doc = r" The HTTP status selected for this response."]
        status: http::StatusCode,
        #[doc = r" The response body."]
        body: types::Error,
    },
}

impl ::std::convert::From<GetItemErrorResponse>
    for ::progenitor_server::ServerError<GetItemErrorResponse>
{
    fn from(body: GetItemErrorResponse) -> Self {
        ::progenitor_server::ServerError::Api(body)
    }
}

#[doc = "Error type for the `get_item` operation."]
pub type GetItemError = ::progenitor_server::ServerError<GetItemErrorResponse>;
#[doc = "Bundled, typed request for the `get_item` operation."]
#[derive(Debug, Clone)]
pub struct GetItemRequest {
    pub item_id: ::std::string::String,
}

#[doc = "The declared responses represented by `UpdateItemError`."]
#[derive(Debug, Clone)]
pub enum UpdateItemErrorResponse {
    #[doc = "The default response."]
    Default {
        #[doc = r" The HTTP status selected for this response."]
        status: http::StatusCode,
        #[doc = r" The response body."]
        body: types::Error,
    },
}

impl ::std::convert::From<UpdateItemErrorResponse>
    for ::progenitor_server::ServerError<UpdateItemErrorResponse>
{
    fn from(body: UpdateItemErrorResponse) -> Self {
        ::progenitor_server::ServerError::Api(body)
    }
}

#[doc = "Error type for the `update_item` operation."]
pub type UpdateItemError = ::progenitor_server::ServerError<UpdateItemErrorResponse>;
#[doc = "Bundled, typed request for the `update_item` operation."]
#[derive(Debug, Clone)]
pub struct UpdateItemRequest {
    pub item_id: ::std::string::String,
    pub dry_run: Option<bool>,
    pub body: types::Item,
}

#[derive(Debug, Clone, :: serde :: Deserialize)]
pub struct UpdateItemQuery {
    #[serde(rename = "dryRun")]
    pub dry_run: Option<bool>,
}

#[doc = "Error type for the `collide` operation."]
pub type CollideError = ::progenitor_server::ServerError<::std::convert::Infallible>;
#[doc = "Bundled, typed request for the `collide` operation."]
#[derive(Debug, Clone)]
pub struct CollideRequest {
    pub inner: ::std::string::String,
    pub query: Option<::std::string::String>,
    pub tags: ::std::vec::Vec<::std::string::String>,
    pub meta: Option<::std::string::String>,
}

#[derive(Debug, Clone, :: serde :: Deserialize)]
pub struct CollideQuery {
    #[serde(rename = "query")]
    pub query: Option<::std::string::String>,
    #[serde(rename = "tags")]
    #[serde(default)]
    pub tags: ::std::vec::Vec<::std::string::String>,
}

#[doc = "Error type for the `download_blob` operation."]
pub type DownloadBlobError = ::progenitor_server::ServerError<::std::convert::Infallible>;
#[doc = "Bundled, typed request for the `download_blob` operation."]
#[derive(Debug, Clone)]
pub struct DownloadBlobRequest {}
#[doc = "The declared responses represented by `MultiKindResponse`."]
#[derive(Debug, Clone)]
pub enum MultiKindResponse {
    #[doc = "The `200` response."]
    Status200(#[doc = r" The response body."] types::Message),
    #[doc = "The `206` response."]
    Status206(#[doc = r" The raw response body."] bytes::Bytes),
}

#[doc = "The declared responses represented by `MultiKindError`."]
#[derive(Debug, Clone)]
pub enum MultiKindErrorResponse {
    #[doc = "The `401` response."]
    Status401(#[doc = r" The response body."] types::Error),
    #[doc = "A `4XX` response."]
    StatusRange4xx {
        #[doc = r" The HTTP status selected for this response."]
        status: ::progenitor_server::ClassStatus<4u16>,
        #[doc = r" The response body."]
        body: types::Error,
    },
    #[doc = "The default response."]
    Default {
        #[doc = r" The HTTP status selected for this response."]
        status: http::StatusCode,
        #[doc = r" The raw response body."]
        body: bytes::Bytes,
    },
}

impl ::std::convert::From<MultiKindErrorResponse>
    for ::progenitor_server::ServerError<MultiKindErrorResponse>
{
    fn from(body: MultiKindErrorResponse) -> Self {
        ::progenitor_server::ServerError::Api(body)
    }
}

#[doc = "Error type for the `multi_kind` operation."]
pub type MultiKindError = ::progenitor_server::ServerError<MultiKindErrorResponse>;
#[doc = "Bundled, typed request for the `multi_kind` operation."]
#[derive(Debug, Clone)]
pub struct MultiKindRequest {
    pub mode: ::std::string::String,
}

#[doc = "Error type for the `upload_item` operation."]
pub type UploadItemError = ::progenitor_server::ServerError<::std::convert::Infallible>;
#[doc = "Bundled, typed request for the `upload_item` operation."]
#[derive(Debug, Clone)]
pub struct UploadItemRequest {
    pub file: ::std::string::String,
    pub multipart_file_part: Option<::std::string::String>,
    pub body: UploadItemMultipartBody,
}

#[derive(Debug, Clone, :: serde :: Deserialize)]
pub struct UploadItemQuery {
    #[serde(rename = "multipart_file_part")]
    pub multipart_file_part: Option<::std::string::String>,
}

#[doc = "Typed multipart request body for the `upload_item` operation."]
#[derive(Debug, Clone)]
pub struct UploadItemMultipartBody {
    #[doc = "The `attachments` multipart field."]
    pub attachments: ::std::vec::Vec<::progenitor_server::multipart::FilePart>,
    #[doc = "The primary upload."]
    pub file: ::progenitor_server::multipart::FilePart,
    #[doc = "The `legacy` multipart field."]
    pub legacy: ::std::option::Option<::std::string::String>,
    #[doc = "The `metadata` multipart field."]
    pub metadata: ::std::option::Option<::std::string::String>,
    #[doc = "A note stored with the upload."]
    pub note: ::std::option::Option<::std::string::String>,
    #[doc = "The `rating` multipart field."]
    pub rating: i32,
    #[doc = "A field named like the generated request URL local."]
    pub url: ::std::option::Option<::std::string::String>,
}

#[doc = "Error type for the `upload_raw` operation."]
pub type UploadRawError = ::progenitor_server::ServerError<::std::convert::Infallible>;
#[doc = "Bundled, typed request for the `upload_raw` operation."]
#[derive(Debug, Clone)]
pub struct UploadRawRequest {
    pub body: bytes::Bytes,
}

#[doc = "Error type for the `render_rejection` operation."]
pub type RenderRejectionError = ::progenitor_server::ServerError<::std::convert::Infallible>;
#[doc = "Bundled, typed request for the `render_rejection` operation."]
#[derive(Debug, Clone)]
pub struct RenderRejectionRequest {}
#[doc = r" The service trait. Implement it, then mount the generated server"]
#[doc = r" adapter via the `progenitor-server` runtime (or `into_router()`"]
#[doc = r" to compose it into your own axum app)."]
#[::progenitor_server::codegen::async_trait]
pub trait ServerGen: Send + Sync + 'static {
    #[doc = r" Renders a failure outside an operation's declared response types."]
    #[doc = r""]
    #[doc = r" The default preserves the runtime status and plain-text message."]
    #[doc = r" An override may change the body and headers. The server adapter"]
    #[doc = r" preserves [`Rejection::status`](::progenitor_server::Rejection::status)"]
    #[doc = r" after the renderer returns."]
    fn render_rejection(
        &self,
        rejection: ::progenitor_server::Rejection,
    ) -> axum::response::Response {
        ::progenitor_server::codegen::axum::response::IntoResponse::into_response(rejection)
    }

    #[doc = "List items, optionally limited."]
    async fn list_items(
        &self,
        request: ::progenitor_server::Request<ListItemsRequest>,
    ) -> ::std::result::Result<::progenitor_server::Response<types::ItemList>, ListItemsError>;
    #[doc = "Create an item (JSON body, 201 with no content)."]
    async fn create_item(
        &self,
        request: ::progenitor_server::Request<CreateItemRequest>,
    ) -> ::std::result::Result<::progenitor_server::Response<()>, CreateItemError>;
    #[doc = "Fetch one item by id."]
    async fn get_item(
        &self,
        request: ::progenitor_server::Request<GetItemRequest>,
    ) -> ::std::result::Result<::progenitor_server::Response<types::Item>, GetItemError>;
    #[doc = "Update an item (path + query + JSON body)."]
    async fn update_item(
        &self,
        request: ::progenitor_server::Request<UpdateItemRequest>,
    ) -> ::std::result::Result<::progenitor_server::Response<types::Item>, UpdateItemError>;
    #[doc = "Params named like the framework bindings, plus a required array query."]
    async fn collide(
        &self,
        request: ::progenitor_server::Request<CollideRequest>,
    ) -> ::std::result::Result<::progenitor_server::Response<types::Message>, CollideError>;
    #[doc = "Raw octet-stream response body."]
    async fn download_blob(
        &self,
        request: ::progenitor_server::Request<DownloadBlobRequest>,
    ) -> ::std::result::Result<::progenitor_server::Response<bytes::Bytes>, DownloadBlobError>;
    #[doc = "Mixed JSON/raw success and error responses."]
    async fn multi_kind(
        &self,
        request: ::progenitor_server::Request<MultiKindRequest>,
    ) -> ::std::result::Result<::progenitor_server::Response<MultiKindResponse>, MultiKindError>;
    #[doc = "Upload typed multipart fields."]
    async fn upload_item(
        &self,
        request: ::progenitor_server::Request<UploadItemRequest>,
    ) -> ::std::result::Result<::progenitor_server::Response<types::Message>, UploadItemError>;
    #[doc = "Keep schema-less multipart as a raw body."]
    async fn upload_raw(
        &self,
        request: ::progenitor_server::Request<UploadRawRequest>,
    ) -> ::std::result::Result<::progenitor_server::Response<()>, UploadRawError>;
    #[doc = "An operation whose name collides with the service rejection hook."]
    async fn render_rejection_2(
        &self,
        request: ::progenitor_server::Request<RenderRejectionRequest>,
    ) -> ::std::result::Result<::progenitor_server::Response<()>, RenderRejectionError>;
}

#[doc = r" Adapter that turns an implementation of the service trait into an"]
#[doc = r" [`axum::Router`] with every route wired."]
pub struct ServerGenServer<T>(Arc<T>);
impl<T: ServerGen> ServerGenServer<T> {
    #[doc = r" Wrap an implementation."]
    pub fn new(inner: T) -> Self {
        Self(Arc::new(inner))
    }

    #[doc = r" Wrap an already-shared implementation."]
    pub fn from_arc(inner: Arc<T>) -> Self {
        Self(inner)
    }

    #[doc = r" Build the fully-wired router."]
    pub fn into_router(self) -> axum::Router {
        axum::Router::new()
            .route(
                "/items",
                axum::routing::get(Self::list_items_route).post(Self::create_item_route),
            )
            .route(
                "/items/{itemId}",
                axum::routing::get(Self::get_item_route).put(Self::update_item_route),
            )
            .route("/collide/{inner}", axum::routing::get(Self::collide_route))
            .route("/blob", axum::routing::get(Self::download_blob_route))
            .route("/multi/{mode}", axum::routing::get(Self::multi_kind_route))
            .route(
                "/upload/{file}",
                axum::routing::post(Self::upload_item_route),
            )
            .route("/upload-raw", axum::routing::post(Self::upload_raw_route))
            .route(
                "/render-rejection",
                axum::routing::post(Self::render_rejection_route),
            )
            .route(
                "/search",
                axum::routing::get(|| async { http::StatusCode::NOT_IMPLEMENTED }),
            )
            .route(
                "/upgrade",
                axum::routing::get(|| async { http::StatusCode::NOT_IMPLEMENTED }),
            )
            .with_state(self.0)
    }

    fn render_rejection(
        __progenitor_inner: &T,
        rejection: ::progenitor_server::Rejection,
    ) -> axum::response::Response {
        let status = rejection.status();
        let mut response = <T as ServerGen>::render_rejection(__progenitor_inner, rejection);
        *response.status_mut() = status;
        response
    }

    async fn list_items_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        __progenitor_query_extractor: ::std::result::Result<
            ::progenitor_server::Query<ListItemsQuery>,
            ::progenitor_server::Rejection,
        >,
    ) -> axum::response::Response {
        let ::progenitor_server::Query(__progenitor_query) = match __progenitor_query_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let x_trace = match ::progenitor_server::optional_header::<::std::string::String>(
            &__progenitor_meta,
            "x-trace",
        ) {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let message = ListItemsRequest {
            limit: __progenitor_query.limit,
            x_trace,
        };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::list_items(&__progenitor_inner, request).await;
        Self::list_items_respond(&__progenitor_inner, result)
    }

    fn list_items_respond(
        __progenitor_inner: &T,
        result: ::std::result::Result<
            ::progenitor_server::Response<types::ItemList>,
            ListItemsError,
        >,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__headers, __body) = response.into_parts();
                let __status = http::StatusCode::OK;
                ::progenitor_server::respond::json(__status, __headers, &__body)
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api(__body) => match __body {
                    ListItemsErrorResponse::Default {
                        status: __response_status,
                        body: __body,
                    } => {
                        let __status = __response_status;
                        ::progenitor_server::respond::json(
                            __status,
                            http::HeaderMap::new(),
                            &__body,
                        )
                    }
                },
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::log_internal(&__e);
                    ::std::mem::drop(__e);
                    Self::render_rejection(
                        __progenitor_inner,
                        ::progenitor_server::Rejection::internal(),
                    )
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn create_item_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        __progenitor_body_extractor: ::std::result::Result<
            ::progenitor_server::Json<types::Item>,
            ::progenitor_server::Rejection,
        >,
    ) -> axum::response::Response {
        let ::progenitor_server::Json(__progenitor_body) = match __progenitor_body_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let message = CreateItemRequest {
            body: __progenitor_body,
        };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::create_item(&__progenitor_inner, request).await;
        Self::create_item_respond(&__progenitor_inner, result)
    }

    fn create_item_respond(
        __progenitor_inner: &T,
        result: ::std::result::Result<::progenitor_server::Response<()>, CreateItemError>,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__headers, __body) = response.into_parts();
                let __status = http::StatusCode::CREATED;
                {
                    let _ = __body;
                    ::progenitor_server::respond::empty(__status, __headers)
                }
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api(__body) => match __body {
                    CreateItemErrorResponse::Default {
                        status: __response_status,
                        body: __body,
                    } => {
                        let __status = __response_status;
                        ::progenitor_server::respond::json(
                            __status,
                            http::HeaderMap::new(),
                            &__body,
                        )
                    }
                },
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::log_internal(&__e);
                    ::std::mem::drop(__e);
                    Self::render_rejection(
                        __progenitor_inner,
                        ::progenitor_server::Rejection::internal(),
                    )
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn get_item_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        __progenitor_path_extractor: ::std::result::Result<
            ::progenitor_server::Path<::std::string::String>,
            ::progenitor_server::Rejection,
        >,
    ) -> axum::response::Response {
        let ::progenitor_server::Path(item_id) = match __progenitor_path_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let message = GetItemRequest { item_id };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::get_item(&__progenitor_inner, request).await;
        Self::get_item_respond(&__progenitor_inner, result)
    }

    fn get_item_respond(
        __progenitor_inner: &T,
        result: ::std::result::Result<::progenitor_server::Response<types::Item>, GetItemError>,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__headers, __body) = response.into_parts();
                let __status = http::StatusCode::OK;
                ::progenitor_server::respond::json(__status, __headers, &__body)
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api(__body) => match __body {
                    GetItemErrorResponse::Default {
                        status: __response_status,
                        body: __body,
                    } => {
                        let __status = __response_status;
                        ::progenitor_server::respond::json(
                            __status,
                            http::HeaderMap::new(),
                            &__body,
                        )
                    }
                },
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::log_internal(&__e);
                    ::std::mem::drop(__e);
                    Self::render_rejection(
                        __progenitor_inner,
                        ::progenitor_server::Rejection::internal(),
                    )
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn update_item_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        __progenitor_path_extractor: ::std::result::Result<
            ::progenitor_server::Path<::std::string::String>,
            ::progenitor_server::Rejection,
        >,
        __progenitor_query_extractor: ::std::result::Result<
            ::progenitor_server::Query<UpdateItemQuery>,
            ::progenitor_server::Rejection,
        >,
        __progenitor_body_extractor: ::std::result::Result<
            ::progenitor_server::Json<types::Item>,
            ::progenitor_server::Rejection,
        >,
    ) -> axum::response::Response {
        let ::progenitor_server::Path(item_id) = match __progenitor_path_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let ::progenitor_server::Query(__progenitor_query) = match __progenitor_query_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let ::progenitor_server::Json(__progenitor_body) = match __progenitor_body_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let message = UpdateItemRequest {
            item_id,
            dry_run: __progenitor_query.dry_run,
            body: __progenitor_body,
        };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::update_item(&__progenitor_inner, request).await;
        Self::update_item_respond(&__progenitor_inner, result)
    }

    fn update_item_respond(
        __progenitor_inner: &T,
        result: ::std::result::Result<::progenitor_server::Response<types::Item>, UpdateItemError>,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__headers, __body) = response.into_parts();
                let __status = http::StatusCode::OK;
                ::progenitor_server::respond::json(__status, __headers, &__body)
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api(__body) => match __body {
                    UpdateItemErrorResponse::Default {
                        status: __response_status,
                        body: __body,
                    } => {
                        let __status = __response_status;
                        ::progenitor_server::respond::json(
                            __status,
                            http::HeaderMap::new(),
                            &__body,
                        )
                    }
                },
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::log_internal(&__e);
                    ::std::mem::drop(__e);
                    Self::render_rejection(
                        __progenitor_inner,
                        ::progenitor_server::Rejection::internal(),
                    )
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn collide_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        __progenitor_path_extractor: ::std::result::Result<
            ::progenitor_server::Path<::std::string::String>,
            ::progenitor_server::Rejection,
        >,
        __progenitor_query_extractor: ::std::result::Result<
            ::progenitor_server::Query<CollideQuery>,
            ::progenitor_server::Rejection,
        >,
    ) -> axum::response::Response {
        let ::progenitor_server::Path(inner) = match __progenitor_path_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let ::progenitor_server::Query(__progenitor_query) = match __progenitor_query_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let meta = match ::progenitor_server::optional_header::<::std::string::String>(
            &__progenitor_meta,
            "meta",
        ) {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let message = CollideRequest {
            inner,
            query: __progenitor_query.query,
            tags: __progenitor_query.tags,
            meta,
        };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::collide(&__progenitor_inner, request).await;
        Self::collide_respond(&__progenitor_inner, result)
    }

    fn collide_respond(
        __progenitor_inner: &T,
        result: ::std::result::Result<::progenitor_server::Response<types::Message>, CollideError>,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__headers, __body) = response.into_parts();
                let __status = http::StatusCode::OK;
                ::progenitor_server::respond::json(__status, __headers, &__body)
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api(__body) => match __body {},
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::log_internal(&__e);
                    ::std::mem::drop(__e);
                    Self::render_rejection(
                        __progenitor_inner,
                        ::progenitor_server::Rejection::internal(),
                    )
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn download_blob_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
    ) -> axum::response::Response {
        let message = DownloadBlobRequest {};
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::download_blob(&__progenitor_inner, request).await;
        Self::download_blob_respond(&__progenitor_inner, result)
    }

    fn download_blob_respond(
        __progenitor_inner: &T,
        result: ::std::result::Result<
            ::progenitor_server::Response<bytes::Bytes>,
            DownloadBlobError,
        >,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__headers, __body) = response.into_parts();
                let __status = http::StatusCode::OK;
                ::progenitor_server::respond::bytes(
                    __status,
                    __headers,
                    "application/octet-stream",
                    __body,
                )
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api(__body) => match __body {},
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::log_internal(&__e);
                    ::std::mem::drop(__e);
                    Self::render_rejection(
                        __progenitor_inner,
                        ::progenitor_server::Rejection::internal(),
                    )
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn multi_kind_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        __progenitor_path_extractor: ::std::result::Result<
            ::progenitor_server::Path<::std::string::String>,
            ::progenitor_server::Rejection,
        >,
    ) -> axum::response::Response {
        let ::progenitor_server::Path(mode) = match __progenitor_path_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let message = MultiKindRequest { mode };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::multi_kind(&__progenitor_inner, request).await;
        Self::multi_kind_respond(&__progenitor_inner, result)
    }

    fn multi_kind_respond(
        __progenitor_inner: &T,
        result: ::std::result::Result<
            ::progenitor_server::Response<MultiKindResponse>,
            MultiKindError,
        >,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__headers, __body) = response.into_parts();
                match __body {
                    MultiKindResponse::Status200(__body) => {
                        let __status = http::StatusCode::OK;
                        ::progenitor_server::respond::json(__status, __headers, &__body)
                    }
                    MultiKindResponse::Status206(__body) => {
                        let __status = http::StatusCode::PARTIAL_CONTENT;
                        ::progenitor_server::respond::bytes(
                            __status,
                            __headers,
                            "application/octet-stream",
                            __body,
                        )
                    }
                }
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api(__body) => match __body {
                    MultiKindErrorResponse::Status401(__body) => {
                        let __status = http::StatusCode::UNAUTHORIZED;
                        ::progenitor_server::respond::json(
                            __status,
                            http::HeaderMap::new(),
                            &__body,
                        )
                    }
                    MultiKindErrorResponse::StatusRange4xx {
                        status: __response_status,
                        body: __body,
                    } => {
                        let __status = __response_status.get();
                        ::progenitor_server::respond::json(
                            __status,
                            http::HeaderMap::new(),
                            &__body,
                        )
                    }
                    MultiKindErrorResponse::Default {
                        status: __response_status,
                        body: __body,
                    } => {
                        let __status = __response_status;
                        ::progenitor_server::respond::bytes(
                            __status,
                            http::HeaderMap::new(),
                            "text/plain",
                            __body,
                        )
                    }
                },
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::log_internal(&__e);
                    ::std::mem::drop(__e);
                    Self::render_rejection(
                        __progenitor_inner,
                        ::progenitor_server::Rejection::internal(),
                    )
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn upload_item_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        __progenitor_path_extractor: ::std::result::Result<
            ::progenitor_server::Path<::std::string::String>,
            ::progenitor_server::Rejection,
        >,
        __progenitor_query_extractor: ::std::result::Result<
            ::progenitor_server::Query<UploadItemQuery>,
            ::progenitor_server::Rejection,
        >,
        __progenitor_body_extractor: ::std::result::Result<
            ::progenitor_server::Multipart,
            ::progenitor_server::Rejection,
        >,
    ) -> axum::response::Response {
        let ::progenitor_server::Path(file) = match __progenitor_path_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let ::progenitor_server::Query(__progenitor_query) = match __progenitor_query_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let __progenitor_body = {
            let __progenitor_multipart = match __progenitor_body_extractor {
                Ok(value) => value,
                Err(rejection) => {
                    return Self::render_rejection(&__progenitor_inner, rejection);
                }
            };
            let mut __progenitor_parts = match ::progenitor_server::multipart::Parts::collect(
                __progenitor_multipart,
            )
            .await
            {
                Ok(parts) => parts,
                Err(rejection) => {
                    return Self::render_rejection(&__progenitor_inner, rejection);
                }
            };
            let attachments = match __progenitor_parts.repeated_file("attachments") {
                Ok(value) => value,
                Err(rejection) => {
                    return Self::render_rejection(&__progenitor_inner, rejection);
                }
            };
            let file = match __progenitor_parts.required_file("file") {
                Ok(value) => value,
                Err(rejection) => {
                    return Self::render_rejection(&__progenitor_inner, rejection);
                }
            };
            let legacy = match __progenitor_parts.optional_text::<::std::string::String>("legacy") {
                Ok(value) => value,
                Err(rejection) => {
                    return Self::render_rejection(&__progenitor_inner, rejection);
                }
            };
            let metadata =
                match __progenitor_parts.optional_text::<::std::string::String>("metadata") {
                    Ok(value) => value,
                    Err(rejection) => {
                        return Self::render_rejection(&__progenitor_inner, rejection);
                    }
                };
            let note = match __progenitor_parts.optional_text::<::std::string::String>("note") {
                Ok(value) => value,
                Err(rejection) => {
                    return Self::render_rejection(&__progenitor_inner, rejection);
                }
            };
            let rating = match __progenitor_parts.required_text::<i32>("rating") {
                Ok(value) => value,
                Err(rejection) => {
                    return Self::render_rejection(&__progenitor_inner, rejection);
                }
            };
            let url = match __progenitor_parts.optional_text::<::std::string::String>("url") {
                Ok(value) => value,
                Err(rejection) => {
                    return Self::render_rejection(&__progenitor_inner, rejection);
                }
            };
            UploadItemMultipartBody {
                attachments,
                file,
                legacy,
                metadata,
                note,
                rating,
                url,
            }
        };
        let message = UploadItemRequest {
            file,
            multipart_file_part: __progenitor_query.multipart_file_part,
            body: __progenitor_body,
        };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::upload_item(&__progenitor_inner, request).await;
        Self::upload_item_respond(&__progenitor_inner, result)
    }

    fn upload_item_respond(
        __progenitor_inner: &T,
        result: ::std::result::Result<
            ::progenitor_server::Response<types::Message>,
            UploadItemError,
        >,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__headers, __body) = response.into_parts();
                let __status = http::StatusCode::CREATED;
                ::progenitor_server::respond::json(__status, __headers, &__body)
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api(__body) => match __body {},
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::log_internal(&__e);
                    ::std::mem::drop(__e);
                    Self::render_rejection(
                        __progenitor_inner,
                        ::progenitor_server::Rejection::internal(),
                    )
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn upload_raw_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        __progenitor_body_extractor: ::std::result::Result<
            ::progenitor_server::Bytes,
            ::progenitor_server::Rejection,
        >,
    ) -> axum::response::Response {
        let ::progenitor_server::Bytes(__progenitor_body) = match __progenitor_body_extractor {
            Ok(value) => value,
            Err(rejection) => {
                return Self::render_rejection(&__progenitor_inner, rejection);
            }
        };
        let message = UploadRawRequest {
            body: __progenitor_body,
        };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::upload_raw(&__progenitor_inner, request).await;
        Self::upload_raw_respond(&__progenitor_inner, result)
    }

    fn upload_raw_respond(
        __progenitor_inner: &T,
        result: ::std::result::Result<::progenitor_server::Response<()>, UploadRawError>,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__headers, __body) = response.into_parts();
                let __status = http::StatusCode::NO_CONTENT;
                {
                    let _ = __body;
                    ::progenitor_server::respond::empty(__status, __headers)
                }
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api(__body) => match __body {},
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::log_internal(&__e);
                    ::std::mem::drop(__e);
                    Self::render_rejection(
                        __progenitor_inner,
                        ::progenitor_server::Rejection::internal(),
                    )
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn render_rejection_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
    ) -> axum::response::Response {
        let message = RenderRejectionRequest {};
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::render_rejection_2(&__progenitor_inner, request).await;
        Self::render_rejection_respond(&__progenitor_inner, result)
    }

    fn render_rejection_respond(
        __progenitor_inner: &T,
        result: ::std::result::Result<::progenitor_server::Response<()>, RenderRejectionError>,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__headers, __body) = response.into_parts();
                let __status = http::StatusCode::NO_CONTENT;
                {
                    let _ = __body;
                    ::progenitor_server::respond::empty(__status, __headers)
                }
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api(__body) => match __body {},
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::log_internal(&__e);
                    ::std::mem::drop(__e);
                    Self::render_rejection(
                        __progenitor_inner,
                        ::progenitor_server::Rejection::internal(),
                    )
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }
}

impl<T: ServerGen> ::progenitor_server::Service for ServerGenServer<T> {
    fn into_router(self) -> axum::Router {
        ServerGenServer::into_router(self)
    }
}
