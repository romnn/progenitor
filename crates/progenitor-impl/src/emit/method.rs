use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Error, Generator, PreparedIr, Result,
    operation::{
        BodyContentType, DROPSHOT_PAGE_TOKEN_PARAM, HttpMethod, MultipartFieldKind, MultipartSpec,
        OperationMethod, OperationParameterKind, OperationParameterType, OperationResponse,
        OperationResponseKind, OperationResponseStatus, ResponseSide, synth_variant_name,
    },
    util::unique_ident_from,
};

pub(super) struct MethodSigBody {
    pub success: TokenStream,
    pub error: TokenStream,
    pub body: TokenStream,
    /// Definitions for any types synthesised by `method_sig_body` itself —
    /// today this is the per-operation `Status<code>` sum-type enums that
    /// `extract_responses` falls back to when the response set has multiple
    /// distinct kinds. Emitted by the caller alongside the function so the
    /// signature's `Synth("…")` identifier resolves.
    pub extra_types: TokenStream,
}

/// Match-arm pattern for an exact response status: the specific code, the
/// bucket's numeric range, or the catch-all for `default`.
fn status_arm_pattern(status: &OperationResponseStatus) -> TokenStream {
    match status {
        OperationResponseStatus::Code(code) => quote! { #code },
        OperationResponseStatus::Range(r) => {
            let min = r * 100;
            let max = min + 99;
            quote! { #min ..= #max }
        }
        OperationResponseStatus::Default => quote! { _ },
    }
}

/// Pattern to emit in the success-arm `match` for a given status code.
/// In the regular (single-kind) case all 2xx statuses collapse into the
/// catch-all `200 ..= 299` arm; in the synth (multi-kind) case each
/// status gets its own specific arm so we dispatch to the right variant
/// constructor.
fn success_arm_pattern(is_synth: bool, status: &OperationResponseStatus) -> TokenStream {
    if is_synth {
        status_arm_pattern(status)
    } else {
        match status {
            OperationResponseStatus::Code(code) => quote! { #code },
            OperationResponseStatus::Range(_) | OperationResponseStatus::Default => {
                quote! { 200 ..= 299 }
            }
        }
    }
}

/// The status window a non-synth match arm accepts, or `None` for a `default`
/// response (whose arm is the catch-all `_`, so it has no window).
///
/// Deliberately mirrors [`success_arm_pattern`] and [`status_arm_pattern`]
/// rather than reading the status directly: on the success side those collapse
/// *any* `Range` to `200 ..= 299`, so reading `Range(r)` as `r * 100 ..=` would
/// silently widen a 3xx-bucketed success into a range the current code never
/// matched.
fn arm_status_window(status: &OperationResponseStatus, is_success: bool) -> Option<(u16, u16)> {
    match status {
        OperationResponseStatus::Code(code) => Some((*code, *code)),
        OperationResponseStatus::Range(_) if is_success => Some((200, 299)),
        OperationResponseStatus::Range(r) => Some((r * 100, r * 100 + 99)),
        OperationResponseStatus::Default => None,
    }
}

/// The status windows for one side of the response set, or `None` if any member
/// decodes differently from `ResponseValue::from_response`.
///
/// A non-synth side always decodes to a single type — `extract_responses` falls
/// back to a synthesized sum type precisely when the members disagree — so once
/// every member is a `Type` the whole side collapses to a list of windows over
/// one `T`.
fn side_status_windows(items: &[OperationResponse], is_success: bool) -> Option<Vec<(u16, u16)>> {
    items
        .iter()
        .map(|item| {
            matches!(item.typ, OperationResponseKind::Type(_))
                .then(|| arm_status_window(&item.status_code, is_success))
                .flatten()
        })
        .collect()
}

/// One side of an operation's response set, reduced to what the hoisting
/// decision needs to look at.
pub(super) struct ResponseSideShape<'a> {
    pub items: &'a [OperationResponse],
    /// The side collapsed to a synthesized sum type because its members decode
    /// to different Rust types.
    pub is_synth: bool,
    /// The side carries a `default` response, whose arm is the catch-all `_`.
    pub has_default: bool,
}

/// Status windows to hand `Client::__progenitor_response`, in match-arm
/// precedence order: success is tried before error, and anything in neither is
/// `UnexpectedResponse`.
pub(super) struct ResponseWindows {
    pub success: Vec<(u16, u16)>,
    pub error: Vec<(u16, u16)>,
}

/// Whether an operation's response set is the shape
/// `Client::__progenitor_response` implements, and if so the status windows to
/// pass it.
///
/// Conservative by construction: anything with a synthesized sum type, a
/// non-JSON body (`empty`/`stream`/`upgrade` decode differently), or a
/// `default` response (which replaces the catch-all arm the helper's final
/// `else` provides) keeps its inlined `match`.
fn hoistable_response_shape(
    success: &ResponseSideShape<'_>,
    error: &ResponseSideShape<'_>,
) -> Option<ResponseWindows> {
    if success.is_synth || error.is_synth || success.has_default || error.has_default {
        return None;
    }

    let success_windows = side_status_windows(success.items, true)?;
    if success_windows.is_empty() {
        return None;
    }

    Some(ResponseWindows {
        success: success_windows,
        error: side_status_windows(error.items, false)?,
    })
}

/// Generate the per-arm decode expression that pulls the response body
/// into a variant of a synthesized response/error enum. The function
/// signature uses `Result<ResponseValue<#enum>, Error<#enum>>` so the
/// per-arm result has to be `ResponseValue<#enum>` (success) or
/// `Err(Error::ErrorResponse(ResponseValue<#enum>))` (error). Each
/// inner-kind branch leverages `ResponseValue::map` (which is
/// infallible but typed as `Result<_, E>`) so `?` threads through the
/// surrounding async block's error type.
fn synth_decode_arm(
    enum_name: &str,
    status: &OperationResponseStatus,
    payload: &OperationResponseKind,
    payload_ident: Option<&TokenStream>,
    response_ident: &proc_macro2::Ident,
    is_error: bool,
) -> TokenStream {
    let enum_ident = format_ident!("{}", enum_name);
    let variant_ident = format_ident!("{}", synth_variant_name(status));

    // Wrap the original kind's decode into a `ResponseValue<#enum>` whose
    // inner value is the right variant constructor. `ResponseValue::map`
    // is infallible but returns `Result<_, E>` so the `?` threads through
    // the surrounding async block's error type without an extra branch.
    //
    // `from_response` and `upgrade` need a turbofish — their return type
    // depends on a `T` the surrounding code can't infer once we collapse
    // the result through the variant constructor.
    let wrap_variant = match payload {
        OperationResponseKind::Type(_) => {
            let ty = payload_ident
                .expect("Type payload requires an ident")
                .clone();
            quote! {
                ResponseValue::<#ty>::from_response(#response_ident)
                    .await?
                    .map(|inner| #enum_ident::#variant_ident(inner))
            }
        }
        OperationResponseKind::None => quote! {
            ResponseValue::empty(#response_ident)
                .map(|()| #enum_ident::#variant_ident)
        },
        OperationResponseKind::Raw => quote! {
            ResponseValue::stream(#response_ident)
                .map(|inner| #enum_ident::#variant_ident(inner))
        },
        OperationResponseKind::Upgrade => quote! {
            ResponseValue::<::reqwest::Upgraded>::upgrade(#response_ident)
                .await?
                .map(|inner| #enum_ident::#variant_ident(inner))
        },
        OperationResponseKind::Synth(_) => {
            unreachable!("Synth kinds cannot themselves contain a synth variant")
        }
    };

    if is_error {
        quote! { Err(Error::ErrorResponse(#wrap_variant)) }
    } else {
        // Success arms must produce `Result<ResponseValue<#enum>, _>`.
        // `wrap_variant` already evaluated to `ResponseValue<#enum>` via
        // the trailing `?`, so wrap it in `Ok(...)` here.
        quote! { Ok(#wrap_variant) }
    }
}

pub(super) fn make_doc_comment(method: &OperationMethod) -> String {
    let mut buf = String::new();

    if let Some(summary) = &method.summary {
        buf.push_str(summary.trim_end_matches(['.', ',']));
        buf.push_str("\n\n");
    }
    if let Some(description) = &method.description {
        buf.push_str(description);
        buf.push_str("\n\n");
    }

    buf.push_str(&format!(
        "Sends a `{}` request to `{}`\n\n",
        method.method.as_str().to_ascii_uppercase(),
        method.path,
    ));

    if method
        .params
        .iter()
        .filter(|param| param.description.is_some())
        .count()
        > 0
    {
        buf.push_str("Arguments:\n");
        for param in &method.params {
            buf.push_str(&format!("- `{}`", param.name));
            if let Some(description) = &param.description {
                buf.push_str(": ");
                buf.push_str(description);
            }
            buf.push('\n');
        }
    }

    crate::util::neutralize_doc_fences(&buf)
}

pub(super) fn make_stream_doc_comment(method: &OperationMethod) -> String {
    let mut buf = String::new();

    if let Some(summary) = &method.summary {
        buf.push_str(summary.trim_end_matches(['.', ',']));
        buf.push_str(" as a Stream\n\n");
    }
    if let Some(description) = &method.description {
        buf.push_str(description);
        buf.push_str("\n\n");
    }

    buf.push_str(&format!(
        "Sends repeated `{}` requests to `{}` until there are no more results.\n\n",
        method.method.as_str().to_ascii_uppercase(),
        method.path,
    ));

    if method
        .params
        .iter()
        .filter(|param| param.api_name != DROPSHOT_PAGE_TOKEN_PARAM)
        .filter(|param| param.description.is_some())
        .count()
        > 0
    {
        buf.push_str("Arguments:\n");
        for param in &method.params {
            if param.api_name == DROPSHOT_PAGE_TOKEN_PARAM {
                continue;
            }

            buf.push_str(&format!("- `{}`", param.name));
            if let Some(description) = &param.description {
                buf.push_str(": ");
                buf.push_str(description);
            }
            buf.push('\n');
        }
    }

    crate::util::neutralize_doc_fences(&buf)
}

impl Generator {
    /// The one inherent `Client` method that runs the hook/execute/hook
    /// sequence shared by every operation.
    ///
    /// This is emitted once per client rather than inlined into each
    /// operation, and the reason is compile time rather than tidiness.
    /// [`ClientHooks::pre`], [`ClientHooks::post`] and [`ClientHooks::exec`]
    /// are `async fn`s in a trait, so each *call site* introduces its own
    /// opaque return type for `rustc` to infer and check. Inlined, a spec with
    /// N operations pays for 3N of those; hoisted, it pays for 3. On a large
    /// spec that is the difference between most of the client's check time and
    /// a third of it.
    ///
    /// It must be an inherent method on the generated `Client` — not a
    /// generic helper in `progenitor-client` — to keep the auto-ref
    /// specialization that lets a consumer override a hook: resolving
    /// `self.pre(…)` with `self: &Client` prefers an `impl ClientHooks for
    /// Client` when one exists and falls back to the blanket `impl … for
    /// &Client` otherwise. Behind a `C: ClientHooks` bound that choice would
    /// already have been made, and the consumer's override would be skipped.
    pub(crate) fn dispatch_method(&self, has_inner: bool) -> TokenStream {
        let request_ident = format_ident!("request");
        let result_ident = format_ident!("result");

        // Mirrors the `inner` token in `method_sig_body`, but `self` here is
        // always the `Client` itself rather than a builder's borrow of it.
        let inner = if has_inner {
            quote! { &self.inner, }
        } else {
            quote! {}
        };
        let pre_hook = self.settings.pre_hook.as_ref().map(|hook| {
            quote! {
                (#hook)(#inner &#request_ident);
            }
        });
        let pre_hook_async = self.settings.pre_hook_async.as_ref().map(|hook| {
            quote! {
                match (#hook)(#inner &mut #request_ident).await {
                    Ok(_) => (),
                    Err(e) => return Err(Error::Custom(e.to_string())),
                }
            }
        });
        let post_hook = self.settings.post_hook.as_ref().map(|hook| {
            quote! {
                (#hook)(#inner &#result_ident);
            }
        });
        let post_hook_async = self.settings.post_hook_async.as_ref().map(|hook| {
            quote! {
                match (#hook)(#inner &#result_ident).await {
                    Ok(_) => (),
                    Err(e) => return Err(Error::Custom(e.to_string())),
                }
            }
        });

        quote! {
            /// Run the request through the pre/post hooks and
            /// [`ClientHooks::exec`], yielding the raw response.
            #[doc(hidden)]
            #[allow(dead_code, clippy::all)]
            pub(crate) async fn __progenitor_dispatch<E>(
                &self,
                #[allow(unused_mut)]
                mut #request_ident: ::reqwest::Request,
                info: &OperationInfo,
            ) -> ::std::result::Result<::reqwest::Response, Error<E>> {
                #pre_hook
                #pre_hook_async
                self.pre(&mut #request_ident, info).await?;

                let #result_ident = self.exec(#request_ident, info).await;

                self.post(&#result_ident, info).await?;
                #post_hook_async
                #post_hook

                ::std::result::Result::Ok(#result_ident?)
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
                #request_ident: ::reqwest::Request,
                info: &OperationInfo,
                success: &[(u16, u16)],
                error: &[(u16, u16)],
            ) -> ::std::result::Result<ResponseValue<T>, Error<E>>
            where
                T: ::serde::de::DeserializeOwned,
                E: ::serde::de::DeserializeOwned,
            {
                let response = self.__progenitor_dispatch(#request_ident, info).await?;
                let status = response.status().as_u16();
                let matches_window = |windows: &[(u16, u16)]| {
                    windows.iter().any(|&(low, high)| status >= low && status <= high)
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
    }

    /// Common code generation between positional and builder interface-styles.
    /// Returns a struct with the success and error types and the core body
    /// implementation that marshals arguments and executes the request.
    pub(super) fn method_sig_body(
        &self,
        prepared: &PreparedIr,
        method: &OperationMethod,
        client_type: TokenStream,
        client_value: TokenStream,
    ) -> Result<MethodSigBody> {
        let param_names = method
            .params
            .iter()
            .map(|param| format_ident!("{}", param.name))
            .collect::<Vec<_>>();

        // Generate a unique Ident for internal variables
        let url_ident = unique_ident_from("url", &param_names);
        let request_ident = unique_ident_from("request", &param_names);
        let response_ident = unique_ident_from("response", &param_names);

        // Generate code for query parameters.
        let query_params = method
            .params
            .iter()
            .filter_map(|param| match &param.kind {
                OperationParameterKind::Query { deep_object, .. } => {
                    let qn = &param.api_name;
                    let qn_ident = format_ident!("{}", &param.name);
                    Some(if *deep_object {
                        quote! {
                            &progenitor_client::DeepObjectQuery::new(#qn, &#qn_ident)
                        }
                    } else {
                        quote! {
                            &progenitor_client::QueryParam::new(#qn, &#qn_ident)
                        }
                    })
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let cookies = method
            .params
            .iter()
            .filter_map(|param| match &param.kind {
                OperationParameterKind::Cookie { .. } => {
                    let cookie_name = &param.api_name;
                    let cookie_ident = format_ident!("{}", &param.name);
                    let cookie = if param.optional {
                        quote! {
                            if let Some(value) = #cookie_ident {
                                cookie_header_values.push(format!(
                                    "{}={}",
                                    #cookie_name,
                                    value,
                                ));
                            }
                        }
                    } else {
                        quote! {
                            cookie_header_values.push(format!(
                                "{}={}",
                                #cookie_name,
                                #cookie_ident,
                            ));
                        }
                    };
                    Some(cookie)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let headers = method
            .params
            .iter()
            .filter_map(|param| match &param.kind {
                OperationParameterKind::Header { .. } => {
                    let hn = &param.api_name;
                    let hn_ident = format_ident!("{}", &param.name);
                    let res = if param.optional {
                        quote! {
                            if let Some(value) = #hn_ident {
                                header_map.append(
                                    #hn,
                                    value.to_string().try_into()?
                                );
                            }
                        }
                    } else {
                        quote! {
                            header_map.append(
                                #hn,
                                #hn_ident.to_string().try_into()?
                            );
                        }
                    };
                    Some(res)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let headers_size = headers.len() + 1;
        let cookies_build = if cookies.is_empty() {
            quote! {}
        } else {
            quote! {
                let mut cookie_header_values = Vec::new();
                #(#cookies)*
                if !cookie_header_values.is_empty() {
                    header_map.append(
                        ::reqwest::header::COOKIE,
                        cookie_header_values.join("; ").try_into()?
                    );
                }
            }
        };
        let headers_build = quote! {
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(#headers_size);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(#client_type::api_version()),
            );

            #(#headers)*
            #cookies_build
        };

        let headers_use = quote! {
            .headers(header_map)
        };

        let websock_hdrs = if method.dropshot_websocket {
            quote! {
                .header(::reqwest::header::CONNECTION, "Upgrade")
                .header(::reqwest::header::UPGRADE, "websocket")
                .header(::reqwest::header::SEC_WEBSOCKET_VERSION, "13")
                .header(
                    ::reqwest::header::SEC_WEBSOCKET_KEY,
                    ::base64::Engine::encode(
                        &::base64::engine::general_purpose::STANDARD,
                        ::rand::random::<[u8; 16]>(),
                    )
                )
            }
        } else {
            quote! {}
        };

        // Generate the path rename map; then use it to generate code for
        // assigning the path parameters to the `url` variable.
        let url_renames = method
            .params
            .iter()
            .filter_map(|param| match &param.kind {
                OperationParameterKind::Path => Some((&param.api_name, &param.name)),
                _ => None,
            })
            .collect();

        let url_path = method.path.compile(url_renames, client_value.clone());
        let url_path = quote! {
            let #url_ident = #url_path;
        };

        // Generate code to handle the body param.
        let body_func = method
            .params
            .iter()
            .filter_map(|param| {
                match (&param.kind, &param.typ) {
                    (
                        OperationParameterKind::Body(BodyContentType::OctetStream),
                        OperationParameterType::RawBody,
                    ) => Some(quote! {
                        // Set the content type (this is handled by helper
                        // functions for other MIME types).
                        .header(
                            ::reqwest::header::CONTENT_TYPE,
                            ::reqwest::header::HeaderValue::from_static("application/octet-stream"),
                        )
                        .body(body)
                    }),
                    (
                        OperationParameterKind::Body(
                            BodyContentType::Text(mime_type) | BodyContentType::Raw(mime_type),
                        ),
                        OperationParameterType::RawBody,
                    ) => Some(quote! {
                        // Set the content type (this is handled by helper
                        // functions for other MIME types).
                        .header(
                            ::reqwest::header::CONTENT_TYPE,
                            ::reqwest::header::HeaderValue::from_static(#mime_type),
                        )
                        .body(body)
                    }),
                    (
                        OperationParameterKind::Body(BodyContentType::Json),
                        OperationParameterType::Type(_),
                    ) => Some(quote! {
                        // Serialization errors are deferred.
                        .json(&body)
                    }),
                    (
                        OperationParameterKind::Body(BodyContentType::FormUrlencoded),
                        OperationParameterType::Type(_),
                    ) => Some(quote! {
                        // This uses progenitor_client::RequestBuilderExt which
                        // returns an error in the case of a serialization failure.
                        .form_urlencoded(&body)?
                    }),
                    (
                        OperationParameterKind::Body(BodyContentType::Multipart),
                        OperationParameterType::Multipart(spec),
                    ) => {
                        let form = self.multipart_form_expr(spec);
                        Some(quote! {
                            .multipart(#form)
                        })
                    }
                    (OperationParameterKind::Body(_), _) => {
                        unreachable!("invalid body kind/type combination")
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        // ... and there can be at most one body.
        assert!(body_func.len() <= 1);

        let (success_response_items, response_type) =
            self.extract_responses(prepared, method, ResponseSide::Success);

        let success_synth_name = match &response_type {
            OperationResponseKind::Synth(name) => Some(name.clone()),
            _ => None,
        };

        let success_response_matches = success_response_items.iter().map(|response| {
            let pat = success_arm_pattern(success_synth_name.is_some(), &response.status_code);
            let decode = if let Some(enum_name) = &success_synth_name {
                let payload_ident = match &response.typ {
                    OperationResponseKind::Type(type_id) => {
                        Some(self.type_space.get_type(type_id).unwrap().ident())
                    }
                    _ => None,
                };
                synth_decode_arm(
                    enum_name,
                    &response.status_code,
                    &response.typ,
                    payload_ident.as_ref(),
                    &response_ident,
                    false,
                )
            } else {
                match &response.typ {
                    OperationResponseKind::Type(_) => {
                        quote! {
                            ResponseValue::from_response(#response_ident).await
                        }
                    }
                    OperationResponseKind::None => {
                        quote! {
                            Ok(ResponseValue::empty(#response_ident))
                        }
                    }
                    OperationResponseKind::Raw => {
                        quote! {
                            Ok(ResponseValue::stream(#response_ident))
                        }
                    }
                    OperationResponseKind::Upgrade => {
                        quote! {
                            ResponseValue::upgrade(#response_ident).await
                        }
                    }
                    OperationResponseKind::Synth(_) => {
                        unreachable!("Synth never appears in per-item typ")
                    }
                }
            };

            quote! { #pat => { #decode } }
        });

        // Errors...
        let (error_response_items, error_type) =
            self.extract_responses(prepared, method, ResponseSide::Error);

        if let Some(response) = error_response_items.iter().find(|response| {
            matches!(response.typ, OperationResponseKind::Upgrade)
                && response.status_code != OperationResponseStatus::Default
        }) {
            return Err(Error::UnexpectedFormat(format!(
                "upgrade operations with non-default error responses are not supported: {:?}",
                response.status_code
            )));
        }

        let error_synth_name = match &error_type {
            OperationResponseKind::Synth(name) => Some(name.clone()),
            _ => None,
        };

        let error_response_matches = error_response_items.iter().map(|response| {
            let pat = status_arm_pattern(&response.status_code);

            let decode = if let Some(enum_name) = &error_synth_name {
                let payload_ident = match &response.typ {
                    OperationResponseKind::Type(type_id) => {
                        Some(self.type_space.get_type(type_id).unwrap().ident())
                    }
                    _ => None,
                };
                synth_decode_arm(
                    enum_name,
                    &response.status_code,
                    &response.typ,
                    payload_ident.as_ref(),
                    &response_ident,
                    true,
                )
            } else {
                match &response.typ {
                    OperationResponseKind::Type(_) => {
                        quote! {
                            Err(Error::ErrorResponse(
                                ResponseValue::from_response(#response_ident)
                                    .await?
                            ))
                        }
                    }
                    OperationResponseKind::None => {
                        quote! {
                            Err(Error::ErrorResponse(
                                ResponseValue::empty(#response_ident)
                            ))
                        }
                    }
                    OperationResponseKind::Raw => {
                        quote! {
                            Err(Error::ErrorResponse(
                                ResponseValue::stream(#response_ident)
                            ))
                        }
                    }
                    OperationResponseKind::Upgrade => {
                        // Handled by the catch-all upgrade arm below; emit
                        // no status arm at all (an empty-bodied arm would
                        // not type-check against the match's result type).
                        return quote! {};
                    }
                    OperationResponseKind::Synth(_) => {
                        unreachable!("Synth never appears in per-item typ")
                    }
                }
            };

            quote! { #pat => { #decode } }
        });

        let accept_header = matches!(
            (&response_type, &error_type),
            (OperationResponseKind::Type(_), _)
                | (OperationResponseKind::None, OperationResponseKind::Type(_))
        )
        .then(|| {
            quote! {
                    .header(
                        ::reqwest::header::ACCEPT,
                        ::reqwest::header::HeaderValue::from_static(
                            "application/json",
                        ),
                    )
            }
        });

        // Generate the catch-all case for other statuses. If the operation
        // specifies a default response on either side it already produced
        // a `_ => …` arm (success side via the synth pattern, error side
        // via the `Default` status pattern); emitting another would yield
        // duplicate match arms and `rustc` would reject the source. The
        // prior implementation looked at `method.responses.iter().last()`
        // and relied on the YAML author putting `default:` after every
        // specific code — but `process_operation` chains
        // `operation.responses.default` BEFORE `operation.responses.responses`,
        // so the last entry is always the highest specific code, never
        // `Default`. Check the filtered/sorted items instead.
        let success_has_default = success_response_items
            .iter()
            .any(|item| item.status_code.is_default());
        let error_has_default = error_response_items
            .iter()
            .any(|item| item.status_code.is_default());
        let default_response = if success_has_default || error_has_default {
            quote! {}
        } else {
            quote! { _ => Err(Error::UnexpectedResponse(#response_ident)), }
        };

        let operation_id = &method.operation_id;
        // reqwest::Client only has convenience helpers for the common
        // verbs; OPTIONS and TRACE operations (Box, Kong) go through
        // `request` with an explicit Method.
        let method_call = match method.method {
            HttpMethod::Options => quote! { request(::reqwest::Method::OPTIONS, #url_ident) },
            HttpMethod::Trace => quote! { request(::reqwest::Method::TRACE, #url_ident) },
            _ => {
                let method_func = format_ident!("{}", method.method.as_str());
                quote! { #method_func (#url_ident) }
            }
        };

        // Operations whose response set is the common shape delegate the
        // whole execute-and-decode step to one inherent `Client` method;
        // everything else keeps its inlined `match`. See
        // `hoistable_response_shape` for what qualifies and `dispatch_method`
        // for why this is worth the branch.
        let dispatch_and_decode = match hoistable_response_shape(
            &ResponseSideShape {
                items: &success_response_items,
                is_synth: success_synth_name.is_some(),
                has_default: success_has_default,
            },
            &ResponseSideShape {
                items: &error_response_items,
                is_synth: error_synth_name.is_some(),
                has_default: error_has_default,
            },
        ) {
            Some(windows) => {
                let window_list = |side: Vec<(u16, u16)>| {
                    let entries = side.into_iter().map(|(low, high)| quote! { (#low, #high) });
                    quote! { &[ #(#entries),* ] }
                };
                let success_arg = window_list(windows.success);
                let error_arg = window_list(windows.error);
                quote! {
                    #client_value
                        .__progenitor_response(#request_ident, &info, #success_arg, #error_arg)
                        .await
                }
            }
            None => quote! {
                let #response_ident = #client_value
                    .__progenitor_dispatch(#request_ident, &info)
                    .await?;

                match #response_ident.status().as_u16() {
                    // These will be of the form...
                    // 201 => ResponseValue::from_response(response).await,
                    // 200..299 => ResponseValue::empty(response),
                    // TODO this kind of enumerated response isn't implemented
                    // ... or in the case of an operation with multiple
                    // successful response types...
                    // 200 => {
                    //     ResponseValue::from_response()
                    //         .await?
                    //         .map(OperationXResponse::ResponseTypeA)
                    // }
                    // 201 => {
                    //     ResponseValue::from_response()
                    //         .await?
                    //         .map(OperationXResponse::ResponseTypeB)
                    // }
                    #(#success_response_matches)*

                    // This is almost identical to the success types except
                    // they are wrapped in Error::ErrorResponse...
                    // 400 => {
                    //     Err(Error::ErrorResponse(
                    //         ResponseValue::from_response(response.await?)
                    //     ))
                    // }
                    #(#error_response_matches)*

                    // The default response is either an Error with a known
                    // type if the operation defines a default (as above) or
                    // an Error::UnexpectedResponse...
                    // _ => Err(Error::UnexpectedResponse(response)),
                    #default_response
                }
            },
        };

        let body_impl = quote! {
            #url_path

            #headers_build

            #[allow(unused_mut)]
            let mut #request_ident = #client_value.client
                . #method_call
                #accept_header
                #(#body_func)*
                #( .query(#query_params) )*
                #headers_use
                #websock_hdrs
                .build()?;

            let info = OperationInfo {
                operation_id: #operation_id,
            };

            #dispatch_and_decode
        };

        // Emit per-operation synthesized enum definitions for any side
        // whose `extract_responses` fell through to the multi-kind sum
        // type. They live in the operations module alongside the function
        // (not in `mod types`) because their variant payloads are
        // status-keyed rather than schema-derived.
        let success_enum = match &response_type {
            OperationResponseKind::Synth(name) => {
                Some(self.synth_enum_definition(name, &success_response_items))
            }
            _ => None,
        };
        let error_enum = match &error_type {
            OperationResponseKind::Synth(name) => {
                Some(self.synth_enum_definition(name, &error_response_items))
            }
            _ => None,
        };
        let multipart_body = method.params.iter().find_map(|param| {
            let OperationParameterType::Multipart(spec) = &param.typ else {
                return None;
            };
            Some(self.multipart_body_definition(method, spec, &quote! { FilePart }))
        });
        let extra_types = quote! { #multipart_body #success_enum #error_enum };

        Ok(MethodSigBody {
            success: response_type.into_tokens(&self.type_space),
            error: error_type.into_tokens(&self.type_space),
            body: body_impl,
            extra_types,
        })
    }

    /// Emit the `{Op}MultipartBody` struct for one side of the wire.
    ///
    /// Client and server emit parallel structs from the same
    /// [`MultipartSpec`] — same field names, same layout rules — differing
    /// only in `file_part`, the path of that side's `FilePart` type. Sharing
    /// this function is what keeps the two layouts from drifting.
    pub(crate) fn multipart_body_definition(
        &self,
        method: &OperationMethod,
        spec: &MultipartSpec,
        file_part: &TokenStream,
    ) -> TokenStream {
        let ident = method.multipart_body_ident();
        let doc = format!(
            "Typed multipart request body for the `{}` operation.",
            method.operation_id,
        );
        let fields = spec.fields.iter().map(|field| {
            let name = format_ident!("{}", field.name);
            let description = field
                .description
                .clone()
                .unwrap_or_else(|| format!("The `{}` multipart field.", field.api_name));
            let typ = match &field.kind {
                MultipartFieldKind::File { repeated: true } => {
                    quote! { ::std::vec::Vec<#file_part> }
                }
                MultipartFieldKind::File { repeated: false } if field.required => file_part.clone(),
                MultipartFieldKind::File { repeated: false } => {
                    quote! { ::std::option::Option<#file_part> }
                }
                MultipartFieldKind::Text(type_id) => {
                    let base = self.type_space.get_type(type_id).unwrap().ident();
                    if field.required {
                        base
                    } else {
                        quote! { ::std::option::Option<#base> }
                    }
                }
            };
            quote! {
                #[doc = #description]
                pub #name: #typ,
            }
        });
        quote! {
            #[doc = #doc]
            #[derive(Debug, Clone)]
            pub struct #ident {
                #(#fields)*
            }
        }
    }

    fn multipart_form_expr(&self, spec: &MultipartSpec) -> TokenStream {
        let append_fields = spec.fields.iter().map(|field| {
            let name = format_ident!("{}", field.name);
            let api_name = &field.api_name;
            match (&field.kind, field.required) {
                (MultipartFieldKind::Text(_), true) => quote! {
                    __progenitor_multipart_form = __progenitor_multipart_form
                        .text(#api_name, body.#name.to_string());
                },
                (MultipartFieldKind::Text(_), false) => quote! {
                    if let Some(value) = body.#name {
                        __progenitor_multipart_form = __progenitor_multipart_form
                            .text(#api_name, value.to_string());
                    }
                },
                (MultipartFieldKind::File { repeated: false }, true) => quote! {
                    __progenitor_multipart_form = __progenitor_multipart_form
                        .part(#api_name, self::multipart_file_part(body.#name)?);
                },
                (MultipartFieldKind::File { repeated: false }, false) => quote! {
                    if let Some(value) = body.#name {
                        __progenitor_multipart_form = __progenitor_multipart_form
                            .part(#api_name, self::multipart_file_part(value)?);
                    }
                },
                (MultipartFieldKind::File { repeated: true }, required) => {
                    // Mirrors the server's missing-part rejection: a required
                    // repeated field with zero files would be indistinguishable
                    // from an absent part on the wire.
                    let require_nonempty = required.then(|| {
                        let message =
                            format!("multipart field `{api_name}` requires at least one file");
                        quote! {
                            if body.#name.is_empty() {
                                return Err(Error::InvalidRequest(#message.to_string()));
                            }
                        }
                    });
                    quote! {
                        #require_nonempty
                        for value in body.#name {
                            __progenitor_multipart_form = __progenitor_multipart_form
                                .part(#api_name, self::multipart_file_part(value)?);
                        }
                    }
                }
            }
        });
        quote! {
            {
                let mut __progenitor_multipart_form = ::reqwest::multipart::Form::new();
                #(#append_fields)*
                __progenitor_multipart_form
            }
        }
    }

    /// Emit a synthesized response/error enum. Variants are derived from
    /// the response items' status codes (`Status401`, `Default`, ...) and
    /// the payload type from each item's `typ`:
    /// `Type(X)` → `Variant(<X>)`, `None` → bare unit variant, `Raw` →
    /// `Variant(ByteStream)`, `Upgrade` → `Variant(::reqwest::Upgraded)`.
    fn synth_enum_definition(&self, name: &str, items: &[OperationResponse]) -> TokenStream {
        let enum_ident = format_ident!("{}", name);
        let variants = items.iter().map(|item| {
            let variant_ident = format_ident!("{}", synth_variant_name(&item.status_code));
            match &item.typ {
                OperationResponseKind::Type(type_id) => {
                    let ty = self.type_space.get_type(type_id).unwrap().ident();
                    quote! { #variant_ident(#ty) }
                }
                OperationResponseKind::None => quote! { #variant_ident },
                OperationResponseKind::Raw => quote! { #variant_ident(ByteStream) },
                OperationResponseKind::Upgrade => quote! { #variant_ident(::reqwest::Upgraded) },
                OperationResponseKind::Synth(_) => {
                    unreachable!("Synth kinds cannot themselves contain a synth variant")
                }
            }
        });
        // Intentionally no `#[derive(Debug)]` — `ByteStream` from
        // `progenitor-client` does not implement `Debug` (it wraps a
        // boxed `Stream<Item = ...>` which isn't), and any synthesised
        // enum that mixes typed + raw responses would otherwise fail to
        // compile. Consumers who want Debug can derive it on a wrapper
        // type or pattern-match on variants directly.
        quote! {
            pub enum #enum_ident {
                #(#variants),*
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use quote::quote;

    use crate::operation::{
        HttpMethod, OperationMethod, OperationResponse, OperationResponseKind,
        OperationResponseStatus,
    };
    use crate::{Error, Generator, PreparedIr};

    #[test]
    fn non_default_upgrade_error_returns_generation_error() {
        let method = OperationMethod {
            operation_id: "upgrade".to_string(),
            tags: Vec::new(),
            method: HttpMethod::Get,
            path: crate::template::parse("/").unwrap(),
            summary: None,
            description: None,
            params: Vec::new(),
            responses: vec![
                OperationResponse {
                    status_code: OperationResponseStatus::Code(200),
                    typ: OperationResponseKind::None,
                    schema_name: None,
                    media_type: None,
                    description: None,
                },
                OperationResponse {
                    status_code: OperationResponseStatus::Code(400),
                    typ: OperationResponseKind::Upgrade,
                    schema_name: None,
                    media_type: None,
                    description: None,
                },
            ],
            dropshot_paginated: None,
            dropshot_websocket: true,
        };

        let prepared = PreparedIr {
            raw_methods: Vec::new(),
            schema_supertypes: BTreeMap::new(),
            schema_type_ids: BTreeMap::new(),
        };
        let result = Generator::default().method_sig_body(
            &prepared,
            &method,
            quote! { Self },
            quote! { self },
        );
        let Err(Error::UnexpectedFormat(message)) = result else {
            panic!("expected unsupported upgrade response error");
        };
        assert!(message.contains("non-default error responses"));
    }
}
