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

    ///Error information from a response.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "description": "Error information from a response.",
    ///  "type": "object",
    ///  "required": [
    ///    "message",
    ///    "request_id"
    ///  ],
    ///  "properties": {
    ///    "error_code": {
    ///      "type": "string"
    ///    },
    ///    "message": {
    ///      "type": "string"
    ///    },
    ///    "request_id": {
    ///      "type": "string"
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Error {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub error_code: ::std::option::Option<::std::string::String>,
        pub message: ::std::string::String,
        pub request_id: ::std::string::String,
    }

    ///A single page of results
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    /// {
    ///  "description": "A single page of results",
    ///  "type": "object",
    ///  "required": [
    ///    "items"
    ///  ],
    ///  "properties": {
    ///    "items": {
    ///      "description": "list of items on this page of results",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "integer",
    ///        "format": "uint32",
    ///        "minimum": 0.0
    ///      }
    ///    },
    ///    "next_page": {
    ///      "description": "token used to fetch the next page of results (if any)",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    }
    ///  }
    /// }
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Uint32ResultsPage {
        ///list of items on this page of results
        pub items: ::std::vec::Vec<u32>,
        ///token used to fetch the next page of results (if any)
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub next_page: ::std::option::Option<::std::string::String>,
    }
}

#[derive(Clone, Debug)]
///Client for test_stream_pagination
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
#[allow(clippy::all)]
impl Client {
    ///Sends a `GET` request to `/`
    ///
    ///Arguments:
    /// - `limit`: Maximum number of items returned by a single call
    /// - `page_token`: Token returned by previous call to retrieve the
    ///   subsequent page
    pub async fn paginated_u32s<'a>(
        &'a self,
        limit: ::std::option::Option<::std::num::NonZeroU32>,
        page_token: ::std::option::Option<&'a str>,
    ) -> Result<ResponseValue<types::Uint32ResultsPage>, Error<types::Error>> {
        let url = format!("{}/", self.baseurl,);
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
            .query(&progenitor_client::QueryParam::new("limit", &limit))
            .query(&progenitor_client::QueryParam::new(
                "page_token",
                &page_token,
            ))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "paginated_u32s",
        };
        self.__progenitor_response(
            request,
            &info,
            &[(200u16, 200u16)],
            &[(400u16, 499u16), (500u16, 599u16)],
        )
        .await
    }

    ///Sends repeated `GET` requests to `/` until there are no more results.
    ///
    ///Arguments:
    /// - `limit`: Maximum number of items returned by a single call
    pub fn paginated_u32s_stream<'a>(
        &'a self,
        limit: ::std::option::Option<::std::num::NonZeroU32>,
    ) -> impl futures::Stream<Item = Result<u32, Error<types::Error>>> + Unpin + 'a {
        use futures::StreamExt;
        use futures::TryFutureExt;
        use futures::TryStreamExt;
        self.paginated_u32s(limit, None)
            .map_ok(move |page| {
                let page = page.into_inner();
                let first = futures::stream::iter(page.items).map(Ok);
                let rest = futures::stream::try_unfold(page.next_page, move |state| async move {
                    if state.is_none() {
                        Ok(None)
                    } else {
                        self.paginated_u32s(limit, state.as_deref())
                            .map_ok(|page| {
                                let page = page.into_inner();
                                Some((futures::stream::iter(page.items).map(Ok), page.next_page))
                            })
                            .await
                    }
                })
                .try_flatten();
                first.chain(rest)
            })
            .try_flatten_stream()
            .boxed()
    }
}

/// Items consumers will typically use such as the Client.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
}
