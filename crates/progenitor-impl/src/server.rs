// Copyright 2026 Oxide Computer Company

//! Generation of an axum server stub: a service trait the user implements plus
//! a `{Api}Server<T>` adapter that turns any `T: {Api}` into an [`axum::Router`].
//!
//! This is the server-side analog of the client codegen. It emits the **body**
//! of a `server` module (no `pub mod server` wrapper — the caller wraps it once),
//! mirroring [`Generator::httpmock`](crate::Generator::httpmock). The generated
//! code references the `progenitor-server` runtime via `progenitor_server::*` and
//! `progenitor_server::codegen::*`, and the SDK's own types via `#crate_path`.
//!
//! Unsupported operations (websocket/upgrade and deepObject query parameters)
//! are not given a trait method; instead a `501 Not Implemented` route stub is
//! emitted so routing stays complete and the gap is visible. Response sets
//! whose status is not uniquely implied get server-side status-keyed enums.

use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use typify::TypeDetails;

use crate::{
    Generator, OpenApiDocument, PreparedIr, Result,
    ir::Document,
    operation::{
        BodyContentType, OperationMethod, OperationParameterKind, OperationParameterType,
        OperationResponse, OperationResponseKind, OperationResponseStatus, ResponseSide,
        synth_variant_name,
    },
    operations::responses::response_items_for_side,
    util::{Case, sanitize},
};

/// The token pieces emitted for one operation.
struct ServerOp {
    /// axum routing path (`/pets/{petId}`).
    axum_path: String,
    /// The axum routing constructor for this op's HTTP method (`get`, `post`, …).
    routing_fn: proc_macro2::Ident,
    /// What to pass to `axum::routing::<method>(...)`: `Self::{op}_route` for a
    /// supported op, or a `501` closure for a skipped one.
    handler: TokenStream,
    /// Items emitted at module scope (request/query structs, error alias).
    module_items: TokenStream,
    /// The trait method declaration (empty for skipped ops).
    trait_method: TokenStream,
    /// The route + responder fns on the adapter (empty for skipped ops).
    adapter_fns: TokenStream,
}

impl Generator {
    /// Generate the server-stub module body for an external crate.
    ///
    /// `crate_path` is a valid Rust path to the SDK crate (with `-` → `_`); the
    /// body references the SDK types via `use #crate_path::types;`. Returns the
    /// module body *without* a `pub mod server { … }` wrapper — the caller adds
    /// it exactly once (mirrors [`Generator::httpmock`](crate::Generator::httpmock)).
    pub fn server(&mut self, spec: &OpenApiDocument, crate_path: &str) -> Result<TokenStream> {
        let prepared = self.prepare(spec)?;
        let document = &spec.0;
        self.server_body(&prepared, document, crate_path)
    }

    /// Shared core used by both [`Generator::server`] and `generate_tokens`.
    pub(crate) fn server_body(
        &mut self,
        prepared: &PreparedIr,
        document: &Document,
        crate_path: &str,
    ) -> Result<TokenStream> {
        self.diagnostics.clear();
        let crate_path = crate::util::parse_rust_path(crate_path, "crate path")?;

        let title = {
            let raw = sanitize(&document.info.title, Case::Pascal);
            if raw.is_empty() || raw.starts_with(|c: char| c.is_ascii_digit()) {
                "Api".to_string()
            } else {
                raw
            }
        };
        let trait_ident = format_ident!("{}", title);
        let server_ident = format_ident!("{}Server", title);

        let ops = prepared
            .raw_methods
            .iter()
            .map(|method| self.server_op(method, &trait_ident))
            .collect::<Result<Vec<_>>>()?;

        let module_items = ops.iter().map(|o| &o.module_items);
        let trait_methods = ops.iter().map(|o| &o.trait_method);
        let adapter_fns = ops.iter().map(|o| &o.adapter_fns);

        // Group all operations (supported and skipped) by path so methods that
        // share a path land in one `MethodRouter` — axum panics on two `.route`
        // calls for the same path otherwise.
        let mut by_path: IndexMap<String, Vec<(proc_macro2::Ident, TokenStream)>> = IndexMap::new();
        for op in &ops {
            by_path
                .entry(op.axum_path.clone())
                .or_default()
                .push((op.routing_fn.clone(), op.handler.clone()));
        }
        let route_calls = by_path.iter().map(|(path, methods)| {
            let mut iter = methods.iter();
            let (first_fn, first_handler) = iter.next().unwrap();
            let mut method_router = quote! { axum::routing::#first_fn(#first_handler) };
            for (routing_fn, handler) in iter {
                method_router = quote! { #method_router.#routing_fn(#handler) };
            }
            quote! { .route(#path, #method_router) }
        });

        Ok(quote! {
            use #crate_path::types;
            #[allow(unused_imports)]
            use ::progenitor_server::codegen::*;

            #(#module_items)*

            /// The service trait. Implement it, then mount the generated server
            /// adapter via the `progenitor-server` runtime (or `into_router()`
            /// to compose it into your own axum app).
            #[::progenitor_server::codegen::async_trait]
            pub trait #trait_ident: Send + Sync + 'static {
                #(#trait_methods)*
            }

            /// Adapter that turns an implementation of the service trait into an
            /// [`axum::Router`] with every route wired.
            pub struct #server_ident<T>(Arc<T>);

            impl<T: #trait_ident> #server_ident<T> {
                /// Wrap an implementation.
                pub fn new(inner: T) -> Self {
                    Self(Arc::new(inner))
                }

                /// Wrap an already-shared implementation.
                pub fn from_arc(inner: Arc<T>) -> Self {
                    Self(inner)
                }

                /// Build the fully-wired router.
                pub fn into_router(self) -> axum::Router {
                    axum::Router::new()
                        #(#route_calls)*
                        .with_state(self.0)
                }

                #(#adapter_fns)*
            }

            impl<T: #trait_ident> ::progenitor_server::Service for #server_ident<T> {
                fn into_router(self) -> axum::Router {
                    #server_ident::into_router(self)
                }
            }
        })
    }

    fn server_op(
        &mut self,
        method: &OperationMethod,
        trait_ident: &proc_macro2::Ident,
    ) -> Result<ServerOp> {
        let op_ident = format_ident!("{}", method.operation_id);
        let route_ident = format_ident!("{}_route", method.operation_id);
        let respond_ident = format_ident!("{}_respond", method.operation_id);
        let pascal = sanitize(&method.operation_id, Case::Pascal);
        let request_ident = format_ident!("{}Request", pascal);
        let query_ident = format_ident!("{}Query", pascal);
        let error_ident = format_ident!("{}Error", pascal);

        let axum_path = method.path.as_axum_path();
        let routing_fn = method.method.routing_ident();

        let (success_items, success_kind) = extract_server_responses(method, ResponseSide::Success);
        let (error_items, error_kind) = extract_server_responses(method, ResponseSide::Error);

        // Websocket/upgrade responses and deepObject query parameters cannot be
        // represented by the generated trait yet. Keep their routes visible as
        // 501 stubs instead of silently omitting them.
        let unsupported_reason = if method.dropshot_websocket {
            Some("websocket/upgrade endpoint")
        } else if method.params.iter().any(|param| {
            matches!(
                param.kind,
                OperationParameterKind::Query {
                    deep_object: true,
                    ..
                }
            )
        }) {
            Some("deepObject query parameter")
        } else if matches!(success_kind, OperationResponseKind::Upgrade)
            || success_items
                .iter()
                .chain(error_items.iter())
                .any(|item| matches!(item.typ, OperationResponseKind::Upgrade))
        {
            Some("upgrade response")
        } else {
            None
        };

        if let Some(reason) = unsupported_reason {
            // A skipped op still gets a 501 route so routing stays complete; it
            // gets no trait method. The skip is surfaced at generation time
            // (comments aren't tokens, so it can't ride along in the source).
            let diagnostic = format!(
                "progenitor: server generation skipped operation `{}` ({reason}); \
                 emitting a 501 route stub",
                method.operation_id
            );
            eprintln!("{diagnostic}");
            self.diagnostics.push(diagnostic);
            let handler = quote! {
                || async { http::StatusCode::NOT_IMPLEMENTED }
            };
            return Ok(ServerOp {
                axum_path,
                routing_fn,
                handler,
                module_items: quote! {},
                trait_method: quote! {},
                adapter_fns: quote! {},
            });
        }

        let success_type = if success_items.is_empty() {
            quote! { ::std::convert::Infallible }
        } else {
            self.response_payload_type(&success_kind, SynthSide::Success)
        };
        let error_type = if error_items.is_empty() {
            quote! { ::std::convert::Infallible }
        } else {
            self.response_payload_type(&error_kind, SynthSide::Error)
        };
        let success_respond = self.success_responder(&success_kind, &success_items);
        let error_respond = self.error_responder(&error_kind, &error_items);

        let parts = self.collect_params(method, &query_ident);
        let CollectedParams {
            request_fields,
            query_struct,
            extractor_args,
            header_lets,
            field_inits,
        } = parts;

        let query_struct_item = query_struct.map(|fields| {
            quote! {
                #[derive(Debug, Clone, ::serde::Deserialize)]
                pub struct #query_ident {
                    #fields
                }
            }
        });

        let error_doc = format!("Error type for the `{}` operation.", method.operation_id);
        let request_doc = format!(
            "Bundled, typed request for the `{}` operation.",
            method.operation_id
        );
        let success_synth_enum =
            self.server_synth_enum_definition(&success_kind, &success_items, SynthSide::Success);
        let error_synth_enum =
            self.server_synth_enum_definition(&error_kind, &error_items, SynthSide::Error);

        let module_items = quote! {
            #success_synth_enum
            #error_synth_enum

            #[doc = #error_doc]
            pub type #error_ident = ::progenitor_server::ServerError<#error_type>;

            #[doc = #request_doc]
            #[derive(Debug, Clone)]
            pub struct #request_ident {
                #request_fields
            }

            #query_struct_item
        };

        let doc = method.summary.as_deref().or(method.description.as_deref());
        let doc_attr = doc.map(|d| quote! { #[doc = #d] });

        let trait_method = quote! {
            #doc_attr
            async fn #op_ident(
                &self,
                request: ::progenitor_server::Request<#request_ident>,
            ) -> ::std::result::Result<::progenitor_server::Response<#success_type>, #error_ident>;
        };

        let adapter_fns = quote! {
            async fn #route_ident(
                #(#extractor_args),*
            ) -> axum::response::Response {
                #(#header_lets)*
                let message = #request_ident { #(#field_inits),* };
                let request =
                    ::progenitor_server::Request::from_metadata(__progenitor_meta, message);
                let result =
                    <T as #trait_ident>::#op_ident(&__progenitor_inner, request).await;
                Self::#respond_ident(result)
            }

            fn #respond_ident(
                result: ::std::result::Result<
                    ::progenitor_server::Response<#success_type>,
                    #error_ident,
                >,
            ) -> axum::response::Response {
                match result {
                    Ok(response) => {
                        #success_respond
                    }
                    Err(__error) => match __error {
                        ::progenitor_server::ServerError::Api(__body) => {
                            #error_respond
                        }
                        ::progenitor_server::ServerError::Internal(__e) => {
                            ::progenitor_server::respond::internal(__e)
                        }
                        ::progenitor_server::ServerError::Response(__r) => __r,
                    },
                }
            }
        };

        let handler = quote! { Self::#route_ident };

        Ok(ServerOp {
            axum_path,
            routing_fn,
            handler,
            module_items,
            trait_method,
            adapter_fns,
        })
    }

    /// Map a response kind to the owned `Response<T>` / error payload type.
    /// Upgrade responses are filtered out before this is called.
    fn response_payload_type(&self, kind: &OperationResponseKind, side: SynthSide) -> TokenStream {
        match kind {
            OperationResponseKind::Type(type_id) => {
                self.type_space.get_type(type_id).unwrap().ident()
            }
            OperationResponseKind::None => quote! { () },
            OperationResponseKind::Raw => quote! { bytes::Bytes },
            OperationResponseKind::Synth(name) => {
                let ident = server_synth_ident(name, side);
                quote! { #ident }
            }
            OperationResponseKind::Upgrade => {
                unreachable!("upgrade responses are skipped before payload typing")
            }
        }
    }

    fn success_responder(
        &self,
        kind: &OperationResponseKind,
        items: &[OperationResponse],
    ) -> TokenStream {
        if items.is_empty() {
            quote! {
                let (_, __body) = response.into_parts();
                match __body {}
            }
        } else if let OperationResponseKind::Synth(name) = kind {
            self.synth_success_encoder(name, items)
        } else {
            let status = single_response_status(items);
            let success_encode = self.success_encoder(kind, items);
            quote! {
                let (__headers, __body) = response.into_parts();
                let __status = #status;
                #success_encode
            }
        }
    }

    fn error_responder(
        &self,
        kind: &OperationResponseKind,
        items: &[OperationResponse],
    ) -> TokenStream {
        if items.is_empty() {
            quote! { match __body {} }
        } else if let OperationResponseKind::Synth(name) = kind {
            self.synth_error_encoder(name, items)
        } else {
            let status = single_response_status(items);
            let error_encode = self.error_encoder(kind, items);
            quote! {
                let __status = #status;
                #error_encode
            }
        }
    }

    fn success_encoder(
        &self,
        kind: &OperationResponseKind,
        items: &[OperationResponse],
    ) -> TokenStream {
        match kind {
            OperationResponseKind::Type(_) => {
                quote! { ::progenitor_server::respond::json(__status, __headers, &__body) }
            }
            OperationResponseKind::None => quote! {
                {
                    let _ = __body;
                    ::progenitor_server::respond::empty(__status, __headers)
                }
            },
            OperationResponseKind::Raw => {
                let content_type = raw_content_type(items);
                quote! {
                    ::progenitor_server::respond::bytes(__status, __headers, #content_type, __body)
                }
            }
            OperationResponseKind::Upgrade | OperationResponseKind::Synth(_) => unreachable!(),
        }
    }

    fn error_encoder(
        &self,
        kind: &OperationResponseKind,
        items: &[OperationResponse],
    ) -> TokenStream {
        match kind {
            OperationResponseKind::Type(_) => quote! {
                ::progenitor_server::respond::json(__status, http::HeaderMap::new(), &__body)
            },
            OperationResponseKind::None => quote! {
                {
                    let _ = __body;
                    ::progenitor_server::respond::empty(__status, http::HeaderMap::new())
                }
            },
            OperationResponseKind::Raw => {
                let content_type = raw_content_type(items);
                quote! {
                    ::progenitor_server::respond::bytes(
                        __status, http::HeaderMap::new(), #content_type, __body,
                    )
                }
            }
            OperationResponseKind::Upgrade | OperationResponseKind::Synth(_) => unreachable!(),
        }
    }

    fn server_synth_enum_definition(
        &self,
        kind: &OperationResponseKind,
        items: &[OperationResponse],
        side: SynthSide,
    ) -> Option<TokenStream> {
        let OperationResponseKind::Synth(name) = kind else {
            return None;
        };
        let enum_ident = server_synth_ident(name, side);
        let variants = items.iter().map(|item| self.synth_variant_definition(item));
        let enum_doc = format!("The declared responses represented by `{name}`.");
        let error_from_impl = if matches!(side, SynthSide::Error) {
            Some(quote! {
                impl ::std::convert::From<#enum_ident>
                    for ::progenitor_server::ServerError<#enum_ident>
                {
                    fn from(body: #enum_ident) -> Self {
                        ::progenitor_server::ServerError::Api(body)
                    }
                }
            })
        } else {
            None
        };

        Some(quote! {
            #[doc = #enum_doc]
            #[derive(Debug, Clone)]
            pub enum #enum_ident {
                #(#variants),*
            }

            #error_from_impl
        })
    }

    fn synth_variant_definition(&self, item: &OperationResponse) -> TokenStream {
        let variant_ident = format_ident!("{}", synth_variant_name(&item.status_code));
        let variant_doc = response_variant_doc(&item.status_code);
        let status_type = synth_status_field_type(&item.status_code);
        match &item.typ {
            OperationResponseKind::Type(type_id) => {
                let ty = self.type_space.get_type(type_id).unwrap().ident();
                if let Some(status_type) = status_type {
                    quote! {
                        #[doc = #variant_doc]
                        #variant_ident {
                            /// The HTTP status selected for this response.
                            status: #status_type,
                            /// The response body.
                            body: #ty,
                        }
                    }
                } else {
                    quote! {
                        #[doc = #variant_doc]
                        #variant_ident(
                            /// The response body.
                            #ty
                        )
                    }
                }
            }
            OperationResponseKind::None => {
                if let Some(status_type) = status_type {
                    quote! {
                        #[doc = #variant_doc]
                        #variant_ident {
                            /// The HTTP status selected for this response.
                            status: #status_type,
                        }
                    }
                } else {
                    quote! {
                        #[doc = #variant_doc]
                        #variant_ident
                    }
                }
            }
            OperationResponseKind::Raw => {
                if let Some(status_type) = status_type {
                    quote! {
                        #[doc = #variant_doc]
                        #variant_ident {
                            /// The HTTP status selected for this response.
                            status: #status_type,
                            /// The raw response body.
                            body: bytes::Bytes,
                        }
                    }
                } else {
                    quote! {
                        #[doc = #variant_doc]
                        #variant_ident(
                            /// The raw response body.
                            bytes::Bytes
                        )
                    }
                }
            }
            OperationResponseKind::Upgrade | OperationResponseKind::Synth(_) => {
                unreachable!("upgrade/synth responses are not synth enum items")
            }
        }
    }

    fn synth_success_encoder(&self, name: &str, items: &[OperationResponse]) -> TokenStream {
        let enum_ident = server_synth_ident(name, SynthSide::Success);
        let arms = items.iter().map(|item| {
            let pattern = synth_variant_pattern(&enum_ident, item);
            let status = synth_variant_status_value(item);
            let encode = self.synth_variant_success_encode(item);
            quote! {
                #pattern => {
                    #status
                    #encode
                }
            }
        });
        quote! {
            let (__headers, __body) = response.into_parts();
            match __body {
                #(#arms),*
            }
        }
    }

    fn synth_error_encoder(&self, name: &str, items: &[OperationResponse]) -> TokenStream {
        let enum_ident = server_synth_ident(name, SynthSide::Error);
        let arms = items.iter().map(|item| {
            let pattern = synth_variant_pattern(&enum_ident, item);
            let status = synth_variant_status_value(item);
            let encode = self.synth_variant_error_encode(item);
            quote! {
                #pattern => {
                    #status
                    #encode
                }
            }
        });
        quote! {
            match __body {
                #(#arms),*
            }
        }
    }

    fn synth_variant_success_encode(&self, item: &OperationResponse) -> TokenStream {
        match &item.typ {
            OperationResponseKind::Type(_) => {
                quote! { ::progenitor_server::respond::json(__status, __headers, &__body) }
            }
            OperationResponseKind::None => {
                quote! { ::progenitor_server::respond::empty(__status, __headers) }
            }
            OperationResponseKind::Raw => {
                let content_type = raw_content_type(std::slice::from_ref(item));
                quote! {
                    ::progenitor_server::respond::bytes(__status, __headers, #content_type, __body)
                }
            }
            OperationResponseKind::Upgrade | OperationResponseKind::Synth(_) => unreachable!(),
        }
    }

    fn synth_variant_error_encode(&self, item: &OperationResponse) -> TokenStream {
        match &item.typ {
            OperationResponseKind::Type(_) => {
                quote! {
                    ::progenitor_server::respond::json(__status, http::HeaderMap::new(), &__body)
                }
            }
            OperationResponseKind::None => {
                quote! { ::progenitor_server::respond::empty(__status, http::HeaderMap::new()) }
            }
            OperationResponseKind::Raw => {
                let content_type = raw_content_type(std::slice::from_ref(item));
                quote! {
                    ::progenitor_server::respond::bytes(
                        __status,
                        http::HeaderMap::new(),
                        #content_type,
                        __body,
                    )
                }
            }
            OperationResponseKind::Upgrade | OperationResponseKind::Synth(_) => unreachable!(),
        }
    }

    fn collect_params(
        &self,
        method: &OperationMethod,
        query_ident: &proc_macro2::Ident,
    ) -> CollectedParams {
        let mut request_fields = TokenStream::new();
        let mut query_fields = TokenStream::new();
        let mut has_query = false;
        let mut extractor_args: Vec<TokenStream> = vec![
            quote! { axum::extract::State(__progenitor_inner): axum::extract::State<Arc<T>> },
            quote! { __progenitor_meta: ::progenitor_server::Metadata },
        ];
        let mut body_extractor_arg: Option<TokenStream> = None;
        let mut header_lets: Vec<TokenStream> = Vec::new();
        let mut field_inits: Vec<TokenStream> = Vec::new();

        // Path params, in path-template order, for the (possibly tuple) Path<…>.
        let mut path_idents: Vec<proc_macro2::Ident> = Vec::new();
        let mut path_types: Vec<TokenStream> = Vec::new();
        for wire in method.path.names() {
            if let Some(param) = method
                .params
                .iter()
                .find(|p| matches!(p.kind, OperationParameterKind::Path) && p.api_name == wire)
            {
                let ident = format_ident!("{}", param.name);
                let ty = match &param.typ {
                    OperationParameterType::Type(id) => {
                        self.type_space.get_type(id).unwrap().ident()
                    }
                    OperationParameterType::RawBody => quote! { String },
                };
                let field_ty = ty.clone();
                request_fields.extend(quote! { pub #ident: #field_ty, });
                field_inits.push(quote! { #ident });
                path_idents.push(ident);
                path_types.push(ty);
            }
        }
        match path_idents.len() {
            0 => {}
            1 => {
                let ident = &path_idents[0];
                let ty = &path_types[0];
                extractor_args.push(quote! {
                    ::progenitor_server::Path(#ident): ::progenitor_server::Path<#ty>
                });
            }
            _ => {
                extractor_args.push(quote! {
                    ::progenitor_server::Path((#(#path_idents),*)):
                        ::progenitor_server::Path<(#(#path_types),*)>
                });
            }
        }

        for param in &method.params {
            match &param.kind {
                OperationParameterKind::Path => {}
                OperationParameterKind::Query {
                    deep_object: false, ..
                } => {
                    has_query = true;
                    let ident = format_ident!("{}", param.name);
                    let (field_ty, _optional) = self.owned_field_type(param);
                    let rename = &param.api_name;
                    // The generated client encodes an empty array query param as
                    // *no* key (repeated-key encoding). serde_html_form then sees
                    // a missing key; `Option<T>` already defaults to `None`, but a
                    // required `Vec<T>` would error. `#[serde(default)]` makes an
                    // absent sequence deserialize to an empty collection so an
                    // empty-array request still round-trips.
                    let seq_default = if self.is_sequence_param(param) {
                        quote! { #[serde(default)] }
                    } else {
                        quote! {}
                    };
                    query_fields.extend(quote! {
                        #[serde(rename = #rename)]
                        #seq_default
                        pub #ident: #field_ty,
                    });
                    request_fields.extend(quote! { pub #ident: #field_ty, });
                    field_inits.push(quote! { #ident: __progenitor_query.#ident });
                }
                OperationParameterKind::Header { .. } => {
                    let ident = format_ident!("{}", param.name);
                    let api_name = &param.api_name;
                    // Headers are parsed via FromStr; if the typed header type
                    // doesn't implement it, degrade to the raw `String` value so
                    // the generated route always compiles.
                    let (base, optional) = self.header_base_type(param);
                    let (field_ty, stmt) = if optional {
                        (
                            quote! { Option<#base> },
                            quote! {
                                let #ident = match ::progenitor_server::optional_header::<#base>(
                                    &__progenitor_meta, #api_name,
                                ) {
                                    Ok(value) => value,
                                    Err(rejection) => {
                                        return axum::response::IntoResponse::into_response(rejection);
                                    }
                                };
                            },
                        )
                    } else {
                        (
                            base.clone(),
                            quote! {
                                let #ident = match ::progenitor_server::required_header::<#base>(
                                    &__progenitor_meta, #api_name,
                                ) {
                                    Ok(value) => value,
                                    Err(rejection) => {
                                        return axum::response::IntoResponse::into_response(rejection);
                                    }
                                };
                            },
                        )
                    };
                    request_fields.extend(quote! { pub #ident: #field_ty, });
                    header_lets.push(stmt);
                    field_inits.push(quote! { #ident });
                }
                OperationParameterKind::Cookie { .. } => {
                    let ident = format_ident!("{}", param.name);
                    let api_name = &param.api_name;
                    let (base, optional) = self.header_base_type(param);
                    let (field_ty, stmt) = if optional {
                        (
                            quote! { Option<#base> },
                            quote! {
                                let #ident = match ::progenitor_server::optional_cookie::<#base>(
                                    &__progenitor_meta, #api_name,
                                ) {
                                    Ok(value) => value,
                                    Err(rejection) => {
                                        return axum::response::IntoResponse::into_response(rejection);
                                    }
                                };
                            },
                        )
                    } else {
                        (
                            base.clone(),
                            quote! {
                                let #ident = match ::progenitor_server::required_cookie::<#base>(
                                    &__progenitor_meta, #api_name,
                                ) {
                                    Ok(value) => value,
                                    Err(rejection) => {
                                        return axum::response::IntoResponse::into_response(rejection);
                                    }
                                };
                            },
                        )
                    };
                    request_fields.extend(quote! { pub #ident: #field_ty, });
                    header_lets.push(stmt);
                    field_inits.push(quote! { #ident });
                }
                OperationParameterKind::Body(content_type) => {
                    let (extractor, field_ty) = self.body_extractor(content_type, &param.typ);
                    request_fields.extend(quote! { pub body: #field_ty, });
                    body_extractor_arg = Some(extractor);
                    field_inits.push(quote! { body: __progenitor_body });
                }
                OperationParameterKind::Query {
                    deep_object: true, ..
                } => {}
            }
        }

        let query_struct = if has_query {
            extractor_args.push(quote! {
                ::progenitor_server::Query(__progenitor_query): ::progenitor_server::Query<#query_ident>
            });
            Some(query_fields)
        } else {
            None
        };
        if let Some(extractor) = body_extractor_arg {
            // axum allows at most one body-consuming `FromRequest` extractor,
            // and it must be the final handler argument. Keep all path/query
            // parts extractors before it even when the OpenAPI param order has
            // the body before query params.
            extractor_args.push(extractor);
        }

        CollectedParams {
            request_fields,
            query_struct,
            extractor_args,
            header_lets,
            field_inits,
        }
    }

    /// Whether the param's (`Option`-unwrapped) type is a sequence (`Vec`,
    /// array, or set) — used to decide whether a query field needs
    /// `#[serde(default)]` so an absent (empty-array) key deserializes to an
    /// empty collection.
    fn is_sequence_param(&self, param: &crate::operation::OperationParameter) -> bool {
        let OperationParameterType::Type(type_id) = &param.typ else {
            return false;
        };
        let ty = self.type_space.get_type(type_id).unwrap();
        let details = ty.details();
        matches!(
            details,
            TypeDetails::Vec(_) | TypeDetails::Array(_, _) | TypeDetails::Set(_)
        )
    }

    /// Owned field type for a query/header param, with the optional-unwrap idiom:
    /// `Option<Inner>` when the param is optional (either by `required = false`
    /// or because the schema type is already `Option<T>`), else the bare owned
    /// type.
    fn owned_field_type(
        &self,
        param: &crate::operation::OperationParameter,
    ) -> (TokenStream, bool) {
        match &param.typ {
            OperationParameterType::Type(type_id) => {
                let ty = self.type_space.get_type(type_id).unwrap();
                if param.optional {
                    let ident = ty.ident();
                    (quote! { Option<#ident> }, true)
                } else {
                    (ty.ident(), false)
                }
            }
            OperationParameterType::RawBody => (quote! { String }, false),
        }
    }

    /// The effective base (non-`Option`) type for a header param plus whether it
    /// is optional. Headers are parsed via `FromStr`; if the typed schema type
    /// doesn't implement `FromStr`, degrade to `String` (the raw header value) so
    /// the generated `required_header`/`optional_header` call always compiles.
    fn header_base_type(
        &self,
        param: &crate::operation::OperationParameter,
    ) -> (TokenStream, bool) {
        match &param.typ {
            OperationParameterType::Type(type_id) => {
                // `typ` is already `Option`-unwrapped (and, for non-`Display`
                // types, string-degraded) by lowering, so it is the base type.
                let base = self.type_space.get_type(type_id).unwrap();
                let effective = if base.has_impl(typify::TypeSpaceImpl::FromStr) {
                    base.ident()
                } else {
                    quote! { String }
                };
                (effective, param.optional)
            }
            OperationParameterType::RawBody => (quote! { String }, param.optional),
        }
    }

    fn body_extractor(
        &self,
        content_type: &BodyContentType,
        typ: &OperationParameterType,
    ) -> (TokenStream, TokenStream) {
        match (content_type, typ) {
            (BodyContentType::Json, OperationParameterType::Type(id)) => {
                let ty = self.type_space.get_type(id).unwrap().ident();
                (
                    quote! { ::progenitor_server::Json(__progenitor_body): ::progenitor_server::Json<#ty> },
                    ty,
                )
            }
            (BodyContentType::FormUrlencoded, OperationParameterType::Type(id)) => {
                let ty = self.type_space.get_type(id).unwrap().ident();
                (
                    quote! { ::progenitor_server::Form(__progenitor_body): ::progenitor_server::Form<#ty> },
                    ty,
                )
            }
            (BodyContentType::Text(_), _) => (
                quote! { ::progenitor_server::Text(__progenitor_body): ::progenitor_server::Text },
                quote! { String },
            ),
            // OctetStream, Raw, and any JSON/form body without a schema are
            // exposed as raw bytes (raw passthrough).
            _ => (
                quote! { ::progenitor_server::Bytes(__progenitor_body): ::progenitor_server::Bytes },
                quote! { bytes::Bytes },
            ),
        }
    }
}

struct CollectedParams {
    request_fields: TokenStream,
    query_struct: Option<TokenStream>,
    extractor_args: Vec<TokenStream>,
    header_lets: Vec<TokenStream>,
    field_inits: Vec<TokenStream>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum SynthSide {
    Success,
    Error,
}

fn server_synth_ident(name: &str, side: SynthSide) -> proc_macro2::Ident {
    match side {
        SynthSide::Success => format_ident!("{}", name),
        // Keep `{Op}Error` available as the public `ServerError<E>` alias.
        SynthSide::Error => format_ident!("{}Response", name),
    }
}

fn extract_server_responses(
    method: &OperationMethod,
    side: ResponseSide,
) -> (Vec<OperationResponse>, OperationResponseKind) {
    let items = response_items_for_side(method, side);
    let kind = match items.as_slice() {
        [] => OperationResponseKind::None,
        [item] if matches!(item.status_code, OperationResponseStatus::Code(_)) => item.typ.clone(),
        _ => OperationResponseKind::Synth(format!(
            "{}{}",
            sanitize(&method.operation_id, Case::Pascal),
            match side {
                ResponseSide::Success => "Response",
                ResponseSide::Error => "Error",
            }
        )),
    };
    (items, kind)
}

fn single_response_status(items: &[OperationResponse]) -> TokenStream {
    let [item] = items else {
        unreachable!("plain response kinds contain exactly one item")
    };
    let OperationResponseStatus::Code(code) = item.status_code else {
        unreachable!("plain response kinds use an exact status")
    };
    status_code_tokens(code)
}

fn status_code_tokens(code: u16) -> TokenStream {
    let named = match code {
        100 => Some("CONTINUE"),
        101 => Some("SWITCHING_PROTOCOLS"),
        102 => Some("PROCESSING"),
        103 => Some("EARLY_HINTS"),
        200 => Some("OK"),
        201 => Some("CREATED"),
        202 => Some("ACCEPTED"),
        203 => Some("NON_AUTHORITATIVE_INFORMATION"),
        204 => Some("NO_CONTENT"),
        205 => Some("RESET_CONTENT"),
        206 => Some("PARTIAL_CONTENT"),
        207 => Some("MULTI_STATUS"),
        208 => Some("ALREADY_REPORTED"),
        226 => Some("IM_USED"),
        300 => Some("MULTIPLE_CHOICES"),
        301 => Some("MOVED_PERMANENTLY"),
        302 => Some("FOUND"),
        303 => Some("SEE_OTHER"),
        304 => Some("NOT_MODIFIED"),
        305 => Some("USE_PROXY"),
        307 => Some("TEMPORARY_REDIRECT"),
        308 => Some("PERMANENT_REDIRECT"),
        400 => Some("BAD_REQUEST"),
        401 => Some("UNAUTHORIZED"),
        402 => Some("PAYMENT_REQUIRED"),
        403 => Some("FORBIDDEN"),
        404 => Some("NOT_FOUND"),
        405 => Some("METHOD_NOT_ALLOWED"),
        406 => Some("NOT_ACCEPTABLE"),
        407 => Some("PROXY_AUTHENTICATION_REQUIRED"),
        408 => Some("REQUEST_TIMEOUT"),
        409 => Some("CONFLICT"),
        410 => Some("GONE"),
        411 => Some("LENGTH_REQUIRED"),
        412 => Some("PRECONDITION_FAILED"),
        413 => Some("PAYLOAD_TOO_LARGE"),
        414 => Some("URI_TOO_LONG"),
        415 => Some("UNSUPPORTED_MEDIA_TYPE"),
        416 => Some("RANGE_NOT_SATISFIABLE"),
        417 => Some("EXPECTATION_FAILED"),
        418 => Some("IM_A_TEAPOT"),
        421 => Some("MISDIRECTED_REQUEST"),
        422 => Some("UNPROCESSABLE_ENTITY"),
        423 => Some("LOCKED"),
        424 => Some("FAILED_DEPENDENCY"),
        425 => Some("TOO_EARLY"),
        426 => Some("UPGRADE_REQUIRED"),
        428 => Some("PRECONDITION_REQUIRED"),
        429 => Some("TOO_MANY_REQUESTS"),
        431 => Some("REQUEST_HEADER_FIELDS_TOO_LARGE"),
        451 => Some("UNAVAILABLE_FOR_LEGAL_REASONS"),
        500 => Some("INTERNAL_SERVER_ERROR"),
        501 => Some("NOT_IMPLEMENTED"),
        502 => Some("BAD_GATEWAY"),
        503 => Some("SERVICE_UNAVAILABLE"),
        504 => Some("GATEWAY_TIMEOUT"),
        505 => Some("HTTP_VERSION_NOT_SUPPORTED"),
        506 => Some("VARIANT_ALSO_NEGOTIATES"),
        507 => Some("INSUFFICIENT_STORAGE"),
        508 => Some("LOOP_DETECTED"),
        510 => Some("NOT_EXTENDED"),
        511 => Some("NETWORK_AUTHENTICATION_REQUIRED"),
        _ => None,
    };
    if let Some(named) = named {
        let ident = format_ident!("{named}");
        quote! { http::StatusCode::#ident }
    } else {
        // Exact codes are validated before responder generation, so this keeps
        // the generated expression typed as `StatusCode` without adding a
        // runtime branch to every response.
        quote! {
            http::StatusCode::from_u16(#code)
                .expect("the generator validates declared HTTP response statuses")
        }
    }
}

fn response_variant_doc(status: &OperationResponseStatus) -> String {
    match status {
        OperationResponseStatus::Code(code) => format!("The `{code}` response."),
        OperationResponseStatus::Range(range) => format!("A `{range}XX` response."),
        OperationResponseStatus::Default => "The default response.".to_string(),
    }
}

fn synth_status_field_type(status: &OperationResponseStatus) -> Option<TokenStream> {
    match status {
        OperationResponseStatus::Code(_) => None,
        OperationResponseStatus::Range(range) => {
            Some(quote! { ::progenitor_server::ClassStatus<#range> })
        }
        OperationResponseStatus::Default => Some(quote! { http::StatusCode }),
    }
}

fn synth_variant_pattern(enum_ident: &proc_macro2::Ident, item: &OperationResponse) -> TokenStream {
    let variant_ident = format_ident!("{}", synth_variant_name(&item.status_code));
    let explicit_status = synth_status_field_type(&item.status_code).is_some();
    match (&item.typ, explicit_status) {
        (OperationResponseKind::None, false) => quote! { #enum_ident::#variant_ident },
        (OperationResponseKind::None, true) => {
            quote! { #enum_ident::#variant_ident { status: __response_status } }
        }
        (_, false) => quote! { #enum_ident::#variant_ident(__body) },
        (_, true) => {
            quote! {
                #enum_ident::#variant_ident {
                    status: __response_status,
                    body: __body,
                }
            }
        }
    }
}

fn synth_variant_status_value(item: &OperationResponse) -> TokenStream {
    match item.status_code {
        OperationResponseStatus::Code(code) => {
            let status = status_code_tokens(code);
            quote! { let __status = #status; }
        }
        OperationResponseStatus::Range(_) => {
            quote! { let __status = __response_status.get(); }
        }
        OperationResponseStatus::Default => {
            quote! { let __status = __response_status; }
        }
    }
}

/// The `content-type` for a raw response: the first item's recorded media type,
/// else `application/octet-stream`.
fn raw_content_type(items: &[OperationResponse]) -> String {
    items
        .iter()
        .find_map(|item| item.media_type.clone())
        .unwrap_or_else(|| "application/octet-stream".to_string())
}

#[cfg(test)]
mod tests {
    use super::extract_server_responses;
    use crate::operation::{
        HttpMethod, OperationMethod, OperationResponse, OperationResponseKind,
        OperationResponseStatus, ResponseSide,
    };

    fn method(statuses: impl IntoIterator<Item = OperationResponseStatus>) -> OperationMethod {
        OperationMethod {
            operation_id: "typed_status".to_string(),
            tags: Vec::new(),
            method: HttpMethod::Get,
            path: crate::template::parse("/").unwrap(),
            summary: None,
            description: None,
            params: Vec::new(),
            responses: statuses
                .into_iter()
                .map(|status_code| OperationResponse {
                    status_code,
                    typ: OperationResponseKind::None,
                    schema_name: None,
                    media_type: None,
                    description: None,
                })
                .collect(),
            dropshot_paginated: None,
            dropshot_websocket: false,
        }
    }

    #[test]
    fn server_uses_plain_payload_only_for_one_exact_status() {
        let exact = method([OperationResponseStatus::Code(204)]);
        let (_, exact_kind) = extract_server_responses(&exact, ResponseSide::Success);
        assert_eq!(exact_kind, OperationResponseKind::None);

        let range = method([OperationResponseStatus::Range(2)]);
        let (_, range_kind) = extract_server_responses(&range, ResponseSide::Success);
        assert_eq!(
            range_kind,
            OperationResponseKind::Synth("TypedStatusResponse".to_string())
        );

        let same_payload = method([
            OperationResponseStatus::Code(404),
            OperationResponseStatus::Code(409),
        ]);
        let (_, error_kind) = extract_server_responses(&same_payload, ResponseSide::Error);
        assert_eq!(
            error_kind,
            OperationResponseKind::Synth("TypedStatusError".to_string())
        );
    }
}
