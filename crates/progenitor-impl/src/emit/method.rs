use super::{
    BodyContentType, DROPSHOT_PAGE_TOKEN_PARAM, Error, Generator, HttpMethod, MethodSigBody,
    OperationMethod, OperationParameterKind, OperationParameterType, OperationResponse,
    OperationResponseKind, OperationResponseStatus, PreparedIr, ResponseSide, Result, TokenStream,
    format_ident, quote, success_arm_pattern, synth_decode_arm, synth_variant_name,
    unique_ident_from,
};

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
    /// Common code generation between positional and builder interface-styles.
    /// Returns a struct with the success and error types and the core body
    /// implementation that marshals arguments and executes the request.
    pub(super) fn method_sig_body(
        &self,
        prepared: &PreparedIr,
        method: &OperationMethod,
        client_type: TokenStream,
        client_value: TokenStream,
        has_inner: bool,
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
        let result_ident = unique_ident_from("result", &param_names);

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
        let body_func = method.params.iter().filter_map(|param| {
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
                (OperationParameterKind::Body(_), _) => {
                    unreachable!("invalid body kind/type combination")
                }
                _ => None,
            }
        });
        // ... and there can be at most one body.
        assert!(body_func.clone().count() <= 1);

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
            let pat = match &response.status_code {
                OperationResponseStatus::Code(code) => {
                    quote! { #code }
                }
                OperationResponseStatus::Range(r) => {
                    let min = r * 100;
                    let max = min + 99;
                    quote! { #min ..= #max }
                }

                OperationResponseStatus::Default => {
                    quote! { _ }
                }
            };

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
                        quote! {} // catch-all handled below
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

        let inner = if has_inner {
            quote! { &#client_value.inner, }
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

            #pre_hook
            #pre_hook_async
            #client_value
                .pre(&mut #request_ident, &info)
                .await?;

            let #result_ident = #client_value
                .exec(#request_ident, &info)
                .await;

            #client_value
                .post(&#result_ident, &info)
                .await?;
            #post_hook_async
            #post_hook

            let #response_ident = #result_ident?;

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
        let extra_types = quote! { #success_enum #error_enum };

        Ok(MethodSigBody {
            success: response_type.into_tokens(&self.type_space),
            error: error_type.into_tokens(&self.type_space),
            body: body_impl,
            extra_types,
        })
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
