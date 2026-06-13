use crate::server_gen_client::types;
#[allow(unused_imports)]
use ::progenitor_server::codegen::*;
#[doc = "Error type for the `list_items` operation."]
pub type ListItemsError = ::progenitor_server::ServerError<types::Error>;
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

#[doc = "Error type for the `create_item` operation."]
pub type CreateItemError = ::progenitor_server::ServerError<types::Error>;
#[doc = "Bundled, typed request for the `create_item` operation."]
#[derive(Debug, Clone)]
pub struct CreateItemRequest {
    pub body: types::Item,
}

#[doc = "Error type for the `get_item` operation."]
pub type GetItemError = ::progenitor_server::ServerError<types::Error>;
#[doc = "Bundled, typed request for the `get_item` operation."]
#[derive(Debug, Clone)]
pub struct GetItemRequest {
    pub item_id: ::std::string::String,
}

#[doc = "Error type for the `update_item` operation."]
pub type UpdateItemError = ::progenitor_server::ServerError<types::Error>;
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
pub type CollideError = ::progenitor_server::ServerError<()>;
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
pub type DownloadBlobError = ::progenitor_server::ServerError<()>;
#[doc = "Bundled, typed request for the `download_blob` operation."]
#[derive(Debug, Clone)]
pub struct DownloadBlobRequest {}
#[derive(Debug, Clone)]
pub enum MultiKindResponse {
    Status200(types::Message),
    Status206(bytes::Bytes),
}

impl MultiKindResponse {
    pub fn status(&self) -> http::StatusCode {
        match self {
            MultiKindResponse::Status200(..) => http::StatusCode::from_u16(200u16).unwrap(),
            MultiKindResponse::Status206(..) => http::StatusCode::from_u16(206u16).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MultiKindErrorResponse {
    Status401(types::Error),
    StatusRange4xx {
        status: http::StatusCode,
        body: types::Error,
    },
    Default {
        status: http::StatusCode,
        body: bytes::Bytes,
    },
}

impl MultiKindErrorResponse {
    pub fn status(&self) -> http::StatusCode {
        match self {
            MultiKindErrorResponse::Status401(..) => http::StatusCode::from_u16(401u16).unwrap(),
            MultiKindErrorResponse::StatusRange4xx { status, .. } => *status,
            MultiKindErrorResponse::Default { status, .. } => *status,
        }
    }
}

impl ::std::convert::From<MultiKindErrorResponse>
    for ::progenitor_server::ServerError<MultiKindErrorResponse>
{
    fn from(body: MultiKindErrorResponse) -> Self {
        let status = body.status();
        ::progenitor_server::ServerError::Api { status, body }
    }
}

#[doc = "Error type for the `multi_kind` operation."]
pub type MultiKindError = ::progenitor_server::ServerError<MultiKindErrorResponse>;
#[doc = "Bundled, typed request for the `multi_kind` operation."]
#[derive(Debug, Clone)]
pub struct MultiKindRequest {
    pub mode: ::std::string::String,
}

#[doc = r" The service trait. Implement it, then mount the generated server"]
#[doc = r" adapter via the `progenitor-server` runtime (or `into_router()`"]
#[doc = r" to compose it into your own axum app)."]
#[::progenitor_server::codegen::async_trait]
pub trait ServerGen: Send + Sync + 'static {
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
                "/search",
                axum::routing::get(|| async { http::StatusCode::NOT_IMPLEMENTED }),
            )
            .route(
                "/upgrade",
                axum::routing::get(|| async { http::StatusCode::NOT_IMPLEMENTED }),
            )
            .with_state(self.0)
    }

    async fn list_items_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        ::progenitor_server::Query(__progenitor_query): ::progenitor_server::Query<ListItemsQuery>,
    ) -> axum::response::Response {
        let x_trace = match ::progenitor_server::optional_header::<::std::string::String>(
            &__progenitor_meta,
            "x-trace",
        ) {
            Ok(value) => value,
            Err(rejection) => {
                return axum::response::IntoResponse::into_response(rejection);
            }
        };
        let message = ListItemsRequest {
            limit: __progenitor_query.limit,
            x_trace,
        };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::list_items(&__progenitor_inner, request).await;
        Self::list_items_respond(result)
    }

    fn list_items_respond(
        result: ::std::result::Result<
            ::progenitor_server::Response<types::ItemList>,
            ListItemsError,
        >,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__status_override, __headers, __body) = response.into_parts();
                let __status = match __status_override {
                    Some(status) => status,
                    None => http::StatusCode::from_u16(200u16).unwrap(),
                };
                {
                    let __code = __status.as_u16();
                    if !(__code == 200u16) {
                        return ::progenitor_server::respond::internal(Box::new(
                            ::std::io::Error::new(
                                ::std::io::ErrorKind::Other,
                                ::std::format!(
                                    "operation `{}` returned undeclared {} status {}",
                                    "list_items",
                                    "success",
                                    __status,
                                ),
                            ),
                        ));
                    }
                }
                ::progenitor_server::respond::json(__status, __headers, &__body)
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api {
                    status: __status,
                    body: __body,
                } => {
                    {
                        let __code = __status.as_u16();
                        if __code == 200u16 {
                            return ::progenitor_server::respond::internal(Box::new(
                                ::std::io::Error::new(
                                    ::std::io::ErrorKind::Other,
                                    ::std::format!(
                                        "operation `{}` returned undeclared {} status {}",
                                        "list_items",
                                        "error",
                                        __status,
                                    ),
                                ),
                            ));
                        }
                    }
                    ::progenitor_server::respond::json(__status, http::HeaderMap::new(), &__body)
                }
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::internal(__e)
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn create_item_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        ::progenitor_server::Json(__progenitor_body): ::progenitor_server::Json<types::Item>,
    ) -> axum::response::Response {
        let message = CreateItemRequest {
            body: __progenitor_body,
        };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::create_item(&__progenitor_inner, request).await;
        Self::create_item_respond(result)
    }

    fn create_item_respond(
        result: ::std::result::Result<::progenitor_server::Response<()>, CreateItemError>,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__status_override, __headers, __body) = response.into_parts();
                let __status = match __status_override {
                    Some(status) => status,
                    None => http::StatusCode::from_u16(201u16).unwrap(),
                };
                {
                    let __code = __status.as_u16();
                    if !(__code == 201u16) {
                        return ::progenitor_server::respond::internal(Box::new(
                            ::std::io::Error::new(
                                ::std::io::ErrorKind::Other,
                                ::std::format!(
                                    "operation `{}` returned undeclared {} status {}",
                                    "create_item",
                                    "success",
                                    __status,
                                ),
                            ),
                        ));
                    }
                }
                {
                    let _ = __body;
                    ::progenitor_server::respond::empty(__status, __headers)
                }
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api {
                    status: __status,
                    body: __body,
                } => {
                    {
                        let __code = __status.as_u16();
                        if __code == 201u16 {
                            return ::progenitor_server::respond::internal(Box::new(
                                ::std::io::Error::new(
                                    ::std::io::ErrorKind::Other,
                                    ::std::format!(
                                        "operation `{}` returned undeclared {} status {}",
                                        "create_item",
                                        "error",
                                        __status,
                                    ),
                                ),
                            ));
                        }
                    }
                    ::progenitor_server::respond::json(__status, http::HeaderMap::new(), &__body)
                }
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::internal(__e)
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn get_item_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        ::progenitor_server::Path(item_id): ::progenitor_server::Path<::std::string::String>,
    ) -> axum::response::Response {
        let message = GetItemRequest { item_id };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::get_item(&__progenitor_inner, request).await;
        Self::get_item_respond(result)
    }

    fn get_item_respond(
        result: ::std::result::Result<::progenitor_server::Response<types::Item>, GetItemError>,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__status_override, __headers, __body) = response.into_parts();
                let __status = match __status_override {
                    Some(status) => status,
                    None => http::StatusCode::from_u16(200u16).unwrap(),
                };
                {
                    let __code = __status.as_u16();
                    if !(__code == 200u16) {
                        return ::progenitor_server::respond::internal(Box::new(
                            ::std::io::Error::new(
                                ::std::io::ErrorKind::Other,
                                ::std::format!(
                                    "operation `{}` returned undeclared {} status {}",
                                    "get_item",
                                    "success",
                                    __status,
                                ),
                            ),
                        ));
                    }
                }
                ::progenitor_server::respond::json(__status, __headers, &__body)
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api {
                    status: __status,
                    body: __body,
                } => {
                    {
                        let __code = __status.as_u16();
                        if __code == 200u16 {
                            return ::progenitor_server::respond::internal(Box::new(
                                ::std::io::Error::new(
                                    ::std::io::ErrorKind::Other,
                                    ::std::format!(
                                        "operation `{}` returned undeclared {} status {}",
                                        "get_item",
                                        "error",
                                        __status,
                                    ),
                                ),
                            ));
                        }
                    }
                    ::progenitor_server::respond::json(__status, http::HeaderMap::new(), &__body)
                }
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::internal(__e)
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn update_item_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        ::progenitor_server::Path(item_id): ::progenitor_server::Path<::std::string::String>,
        ::progenitor_server::Query(__progenitor_query): ::progenitor_server::Query<UpdateItemQuery>,
        ::progenitor_server::Json(__progenitor_body): ::progenitor_server::Json<types::Item>,
    ) -> axum::response::Response {
        let message = UpdateItemRequest {
            item_id,
            dry_run: __progenitor_query.dry_run,
            body: __progenitor_body,
        };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::update_item(&__progenitor_inner, request).await;
        Self::update_item_respond(result)
    }

    fn update_item_respond(
        result: ::std::result::Result<::progenitor_server::Response<types::Item>, UpdateItemError>,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__status_override, __headers, __body) = response.into_parts();
                let __status = match __status_override {
                    Some(status) => status,
                    None => http::StatusCode::from_u16(200u16).unwrap(),
                };
                {
                    let __code = __status.as_u16();
                    if !(__code == 200u16) {
                        return ::progenitor_server::respond::internal(Box::new(
                            ::std::io::Error::new(
                                ::std::io::ErrorKind::Other,
                                ::std::format!(
                                    "operation `{}` returned undeclared {} status {}",
                                    "update_item",
                                    "success",
                                    __status,
                                ),
                            ),
                        ));
                    }
                }
                ::progenitor_server::respond::json(__status, __headers, &__body)
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api {
                    status: __status,
                    body: __body,
                } => {
                    {
                        let __code = __status.as_u16();
                        if __code == 200u16 {
                            return ::progenitor_server::respond::internal(Box::new(
                                ::std::io::Error::new(
                                    ::std::io::ErrorKind::Other,
                                    ::std::format!(
                                        "operation `{}` returned undeclared {} status {}",
                                        "update_item",
                                        "error",
                                        __status,
                                    ),
                                ),
                            ));
                        }
                    }
                    ::progenitor_server::respond::json(__status, http::HeaderMap::new(), &__body)
                }
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::internal(__e)
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn collide_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        ::progenitor_server::Path(inner): ::progenitor_server::Path<::std::string::String>,
        ::progenitor_server::Query(__progenitor_query): ::progenitor_server::Query<CollideQuery>,
    ) -> axum::response::Response {
        let meta = match ::progenitor_server::optional_header::<::std::string::String>(
            &__progenitor_meta,
            "meta",
        ) {
            Ok(value) => value,
            Err(rejection) => {
                return axum::response::IntoResponse::into_response(rejection);
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
        Self::collide_respond(result)
    }

    fn collide_respond(
        result: ::std::result::Result<::progenitor_server::Response<types::Message>, CollideError>,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__status_override, __headers, __body) = response.into_parts();
                let __status = match __status_override {
                    Some(status) => status,
                    None => http::StatusCode::from_u16(200u16).unwrap(),
                };
                {
                    let __code = __status.as_u16();
                    if !(__code == 200u16) {
                        return ::progenitor_server::respond::internal(Box::new(
                            ::std::io::Error::new(
                                ::std::io::ErrorKind::Other,
                                ::std::format!(
                                    "operation `{}` returned undeclared {} status {}",
                                    "collide",
                                    "success",
                                    __status,
                                ),
                            ),
                        ));
                    }
                }
                ::progenitor_server::respond::json(__status, __headers, &__body)
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api {
                    status: __status,
                    body: __body,
                } => {
                    {
                        let __code = __status.as_u16();
                        if true {
                            return ::progenitor_server::respond::internal(Box::new(
                                ::std::io::Error::new(
                                    ::std::io::ErrorKind::Other,
                                    ::std::format!(
                                        "operation `{}` returned undeclared {} status {}",
                                        "collide",
                                        "error",
                                        __status,
                                    ),
                                ),
                            ));
                        }
                    }
                    {
                        let _ = __body;
                        ::progenitor_server::respond::empty(__status, http::HeaderMap::new())
                    }
                }
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::internal(__e)
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
        Self::download_blob_respond(result)
    }

    fn download_blob_respond(
        result: ::std::result::Result<
            ::progenitor_server::Response<bytes::Bytes>,
            DownloadBlobError,
        >,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__status_override, __headers, __body) = response.into_parts();
                let __status = match __status_override {
                    Some(status) => status,
                    None => http::StatusCode::from_u16(200u16).unwrap(),
                };
                {
                    let __code = __status.as_u16();
                    if !(__code == 200u16) {
                        return ::progenitor_server::respond::internal(Box::new(
                            ::std::io::Error::new(
                                ::std::io::ErrorKind::Other,
                                ::std::format!(
                                    "operation `{}` returned undeclared {} status {}",
                                    "download_blob",
                                    "success",
                                    __status,
                                ),
                            ),
                        ));
                    }
                }
                ::progenitor_server::respond::bytes(
                    __status,
                    __headers,
                    "application/octet-stream",
                    __body,
                )
            }
            Err(__error) => match __error {
                ::progenitor_server::ServerError::Api {
                    status: __status,
                    body: __body,
                } => {
                    {
                        let __code = __status.as_u16();
                        if true {
                            return ::progenitor_server::respond::internal(Box::new(
                                ::std::io::Error::new(
                                    ::std::io::ErrorKind::Other,
                                    ::std::format!(
                                        "operation `{}` returned undeclared {} status {}",
                                        "download_blob",
                                        "error",
                                        __status,
                                    ),
                                ),
                            ));
                        }
                    }
                    {
                        let _ = __body;
                        ::progenitor_server::respond::empty(__status, http::HeaderMap::new())
                    }
                }
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::internal(__e)
                }
                ::progenitor_server::ServerError::Response(__r) => __r,
            },
        }
    }

    async fn multi_kind_route(
        axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>>,
        __progenitor_meta: ::progenitor_server::Metadata,
        ::progenitor_server::Path(mode): ::progenitor_server::Path<::std::string::String>,
    ) -> axum::response::Response {
        let message = MultiKindRequest { mode };
        let request = ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
        let result = <T as ServerGen>::multi_kind(&__progenitor_inner, request).await;
        Self::multi_kind_respond(result)
    }

    fn multi_kind_respond(
        result: ::std::result::Result<
            ::progenitor_server::Response<MultiKindResponse>,
            MultiKindError,
        >,
    ) -> axum::response::Response {
        match result {
            Ok(response) => {
                let (__status_override, __headers, __body) = response.into_parts();
                match __body {
                    MultiKindResponse::Status200(__body) => {
                        let __status = http::StatusCode::from_u16(200u16).unwrap();
                        if let Some(__override_status) = __status_override {
                            if __override_status != __status {
                                return ::progenitor_server::respond::internal(Box::new(
                                    ::std::io::Error::new(
                                        ::std::io::ErrorKind::Other,
                                        ::std::format!(
                                            "operation `{}` returned conflicting success status \
                                             {} for synth variant status {}",
                                            "multi_kind",
                                            __override_status,
                                            __status,
                                        ),
                                    ),
                                ));
                            }
                        }
                        ::progenitor_server::respond::json(__status, __headers, &__body)
                    }
                    MultiKindResponse::Status206(__body) => {
                        let __status = http::StatusCode::from_u16(206u16).unwrap();
                        if let Some(__override_status) = __status_override {
                            if __override_status != __status {
                                return ::progenitor_server::respond::internal(Box::new(
                                    ::std::io::Error::new(
                                        ::std::io::ErrorKind::Other,
                                        ::std::format!(
                                            "operation `{}` returned conflicting success status \
                                             {} for synth variant status {}",
                                            "multi_kind",
                                            __override_status,
                                            __status,
                                        ),
                                    ),
                                ));
                            }
                        }
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
                ::progenitor_server::ServerError::Api {
                    status: __status,
                    body: __body,
                } => {
                    {
                        let __code = __status.as_u16();
                        if __code == 200u16 || __code == 206u16 {
                            return ::progenitor_server::respond::internal(Box::new(
                                ::std::io::Error::new(
                                    ::std::io::ErrorKind::Other,
                                    ::std::format!(
                                        "operation `{}` returned undeclared {} status {}",
                                        "multi_kind",
                                        "error",
                                        __status,
                                    ),
                                ),
                            ));
                        }
                    }
                    match __body {
                        MultiKindErrorResponse::Status401(__body) => {
                            let __variant_status = http::StatusCode::from_u16(401u16).unwrap();
                            if __variant_status != __status {
                                return ::progenitor_server::respond::internal(Box::new(
                                    ::std::io::Error::new(
                                        ::std::io::ErrorKind::Other,
                                        ::std::format!(
                                            "operation `{}` returned conflicting error status {} \
                                             for synth variant status {}",
                                            "multi_kind",
                                            __status,
                                            __variant_status,
                                        ),
                                    ),
                                ));
                            }
                            ::progenitor_server::respond::json(
                                __status,
                                http::HeaderMap::new(),
                                &__body,
                            )
                        }
                        MultiKindErrorResponse::StatusRange4xx {
                            status: __variant_status,
                            body: __body,
                        } => {
                            {
                                let __code = __variant_status.as_u16();
                                if !matches!(__code, 400u16..=499u16) || __code == 401u16 {
                                    return ::progenitor_server::respond::internal(Box::new(
                                        ::std::io::Error::new(
                                            ::std::io::ErrorKind::Other,
                                            ::std::format!(
                                                "operation `{}` returned status {} with \
                                                 mismatched {} response variant",
                                                "multi_kind",
                                                __variant_status,
                                                "error",
                                            ),
                                        ),
                                    ));
                                }
                            }
                            if __variant_status != __status {
                                return ::progenitor_server::respond::internal(Box::new(
                                    ::std::io::Error::new(
                                        ::std::io::ErrorKind::Other,
                                        ::std::format!(
                                            "operation `{}` returned conflicting error status {} \
                                             for synth variant status {}",
                                            "multi_kind",
                                            __status,
                                            __variant_status,
                                        ),
                                    ),
                                ));
                            }
                            ::progenitor_server::respond::json(
                                __status,
                                http::HeaderMap::new(),
                                &__body,
                            )
                        }
                        MultiKindErrorResponse::Default {
                            status: __variant_status,
                            body: __body,
                        } => {
                            {
                                let __code = __variant_status.as_u16();
                                if __code == 401u16 || matches!(__code, 400u16..=499u16) {
                                    return ::progenitor_server::respond::internal(Box::new(
                                        ::std::io::Error::new(
                                            ::std::io::ErrorKind::Other,
                                            ::std::format!(
                                                "operation `{}` returned status {} with \
                                                 mismatched {} response variant",
                                                "multi_kind",
                                                __variant_status,
                                                "error",
                                            ),
                                        ),
                                    ));
                                }
                            }
                            if __variant_status != __status {
                                return ::progenitor_server::respond::internal(Box::new(
                                    ::std::io::Error::new(
                                        ::std::io::ErrorKind::Other,
                                        ::std::format!(
                                            "operation `{}` returned conflicting error status {} \
                                             for synth variant status {}",
                                            "multi_kind",
                                            __status,
                                            __variant_status,
                                        ),
                                    ),
                                ));
                            }
                            ::progenitor_server::respond::bytes(
                                __status,
                                http::HeaderMap::new(),
                                "text/plain",
                                __body,
                            )
                        }
                    }
                }
                ::progenitor_server::ServerError::Internal(__e) => {
                    ::progenitor_server::respond::internal(__e)
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
