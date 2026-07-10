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
//! emitted so routing stays complete and the gap is visible. Synthesized
//! multi-kind responses get server-side status-keyed enums. See the
//! `07-server-generation.md` plan, especially §6.4/§6.5, for the contract.

use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use typify::TypeDetails;

use crate::{
    Generator, OpenApiDocument, Result,
    ir::Document,
    operation::{
        BodyContentType, OperationMethod, OperationParameterKind,
        OperationParameterType, OperationResponse, OperationResponseKind, OperationResponseStatus,
        ResponseSide, synth_variant_name,
    },
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
        self.server_body(&prepared.raw_methods, document, crate_path)
    }

    /// Shared core used by both [`Generator::server`] (after `prepare`) and
    /// `generate_tokens` (after its inline prepare). Callers MUST have run
    /// `prepare` first so `extract_responses` sees the supertype maps.
    pub(crate) fn server_body(
        &mut self,
        raw_methods: &[OperationMethod],
        document: &Document,
        crate_path: &str,
    ) -> Result<TokenStream> {
        let crate_path = crate::util::parse_crate_path(crate_path)?;

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

        let ops = raw_methods
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
        &self,
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

        let (success_items, success_kind) = self.extract_responses(method, ResponseSide::Success);
        let (error_items, error_kind) = self.extract_responses(method, ResponseSide::Error);

        // Decide whether this operation is supported. Websocket/upgrade and
        // deepObject query parameters get a 501 stub instead of a trait method
        // (see plan §6.4 D-skip / §6.8). Synthesized multi-kind responses are
        // supported below via server-side status-keyed enums, except for any
        // response item that itself needs an HTTP upgrade.
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
            eprintln!(
                "progenitor: server generation skipped operation `{}` ({reason}); \
                 emitting a 501 route stub",
                method.operation_id
            );
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

        let success_type = self.response_payload_type(&success_kind, SynthSide::Success);
        let error_type = self.response_payload_type(&error_kind, SynthSide::Error);
        let default_status = default_success_status(&success_items);
        let success_respond = self.success_responder(
            &method.operation_id,
            &success_kind,
            &success_items,
            default_status,
        );
        let error_respond = self.error_responder(
            &method.operation_id,
            &error_kind,
            &error_items,
            &success_items,
            matches!(success_kind, OperationResponseKind::Synth(_)),
        );

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
                        ::progenitor_server::ServerError::Api { status: __status, body: __body } => {
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
        operation_id: &str,
        kind: &OperationResponseKind,
        items: &[OperationResponse],
        default_status: u16,
    ) -> TokenStream {
        match kind {
            OperationResponseKind::Synth(name) => {
                self.synth_success_encoder(operation_id, name, items)
            }
            _ => {
                let success_status_guard =
                    response_status_guard(operation_id, items, ResponseSide::Success, false, None);
                let success_encode = self.success_encoder(kind, items);
                quote! {
                    let (__status_override, __headers, __body) = response.into_parts();
                    let __status = match __status_override {
                        Some(status) => status,
                        None => http::StatusCode::from_u16(#default_status).unwrap(),
                    };
                    #success_status_guard
                    #success_encode
                }
            }
        }
    }

    fn error_responder(
        &self,
        operation_id: &str,
        kind: &OperationResponseKind,
        items: &[OperationResponse],
        success_items: &[OperationResponse],
        success_is_synth: bool,
    ) -> TokenStream {
        let error_status_guard = response_status_guard(
            operation_id,
            items,
            ResponseSide::Error,
            false,
            Some((success_items, success_is_synth)),
        );
        match kind {
            OperationResponseKind::Synth(name) => {
                let error_encode = self.synth_error_encoder(operation_id, name, items);
                quote! {
                    #error_status_guard
                    #error_encode
                }
            }
            _ => {
                let error_encode = self.error_encoder(kind, items);
                quote! {
                    #error_status_guard
                    #error_encode
                }
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
        let status_arms = items
            .iter()
            .map(|item| synth_variant_status_arm(&enum_ident, item));
        let error_from_impl = if matches!(side, SynthSide::Error) {
            Some(quote! {
                impl ::std::convert::From<#enum_ident>
                    for ::progenitor_server::ServerError<#enum_ident>
                {
                    fn from(body: #enum_ident) -> Self {
                        let status = body.status();
                        ::progenitor_server::ServerError::Api { status, body }
                    }
                }
            })
        } else {
            None
        };

        Some(quote! {
            #[derive(Debug, Clone)]
            pub enum #enum_ident {
                #(#variants),*
            }

            impl #enum_ident {
                pub fn status(&self) -> http::StatusCode {
                    match self {
                        #(#status_arms),*
                    }
                }
            }

            #error_from_impl
        })
    }

    fn synth_variant_definition(&self, item: &OperationResponse) -> TokenStream {
        let variant_ident = format_ident!("{}", synth_variant_name(&item.status_code));
        let status_field = needs_explicit_synth_status(&item.status_code);
        match &item.typ {
            OperationResponseKind::Type(type_id) => {
                let ty = self.type_space.get_type(type_id).unwrap().ident();
                if status_field {
                    quote! { #variant_ident { status: http::StatusCode, body: #ty } }
                } else {
                    quote! { #variant_ident(#ty) }
                }
            }
            OperationResponseKind::None => {
                if status_field {
                    quote! { #variant_ident { status: http::StatusCode } }
                } else {
                    quote! { #variant_ident }
                }
            }
            OperationResponseKind::Raw => {
                if status_field {
                    quote! { #variant_ident { status: http::StatusCode, body: bytes::Bytes } }
                } else {
                    quote! { #variant_ident(bytes::Bytes) }
                }
            }
            OperationResponseKind::Upgrade | OperationResponseKind::Synth(_) => {
                unreachable!("upgrade/synth responses are not synth enum items")
            }
        }
    }

    fn synth_success_encoder(
        &self,
        operation_id: &str,
        name: &str,
        items: &[OperationResponse],
    ) -> TokenStream {
        let enum_ident = server_synth_ident(name, SynthSide::Success);
        let arms = items.iter().enumerate().map(|(index, item)| {
            let pattern = synth_variant_pattern(&enum_ident, item, quote! { __status });
            let status = synth_variant_status_value(item, quote! { __status });
            let status_guard = synth_variant_status_guard(
                operation_id,
                item,
                &items[..index],
                ResponseSide::Success,
                quote! { __status },
            );
            let override_guard = synth_success_override_guard(operation_id);
            let encode = self.synth_variant_success_encode(item);
            quote! {
                #pattern => {
                    #status
                    #status_guard
                    #override_guard
                    #encode
                }
            }
        });
        quote! {
            let (__status_override, __headers, __body) = response.into_parts();
            match __body {
                #(#arms),*
            }
        }
    }

    fn synth_error_encoder(
        &self,
        operation_id: &str,
        name: &str,
        items: &[OperationResponse],
    ) -> TokenStream {
        let enum_ident = server_synth_ident(name, SynthSide::Error);
        let arms = items.iter().enumerate().map(|(index, item)| {
            let pattern = synth_variant_pattern(&enum_ident, item, quote! { __variant_status });
            let variant_status = synth_variant_status_value(item, quote! { __variant_status });
            let status_guard = synth_variant_status_guard(
                operation_id,
                item,
                &items[..index],
                ResponseSide::Error,
                quote! { __variant_status },
            );
            let status_match_guard = synth_error_status_match_guard(operation_id);
            let encode = self.synth_variant_error_encode(item);
            quote! {
                #pattern => {
                    #variant_status
                    #status_guard
                    #status_match_guard
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
                    required,
                    deep_object: false,
                } => {
                    has_query = true;
                    let ident = format_ident!("{}", param.name);
                    let (field_ty, _optional) = self.owned_field_type(param, *required);
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
                OperationParameterKind::Header { required } => {
                    let ident = format_ident!("{}", param.name);
                    let api_name = &param.api_name;
                    // Headers are parsed via FromStr; if the typed header type
                    // doesn't implement it, degrade to the raw `String` value so
                    // the generated route always compiles.
                    let (base, optional) = self.header_base_type(param, *required);
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
                OperationParameterKind::Cookie { required } => {
                    let ident = format_ident!("{}", param.name);
                    let api_name = &param.api_name;
                    let (base, optional) = self.header_base_type(param, *required);
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
        let inner_ty;
        let details = if let TypeDetails::Option(inner) = ty.details() {
            inner_ty = self.type_space.get_type(&inner).unwrap();
            inner_ty.details()
        } else {
            ty.details()
        };
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
        kind_required: bool,
    ) -> (TokenStream, bool) {
        match &param.typ {
            OperationParameterType::Type(type_id) => {
                let ty = self.type_space.get_type(type_id).unwrap();
                if let TypeDetails::Option(inner) = ty.details() {
                    let inner = self.type_space.get_type(&inner).unwrap().ident();
                    (quote! { Option<#inner> }, true)
                } else if !kind_required {
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
        kind_required: bool,
    ) -> (TokenStream, bool) {
        match &param.typ {
            OperationParameterType::Type(type_id) => {
                let ty = self.type_space.get_type(type_id).unwrap();
                let (base, optional) = if let TypeDetails::Option(inner) = ty.details() {
                    (self.type_space.get_type(&inner).unwrap(), true)
                } else {
                    (ty, !kind_required)
                };
                let effective = if base.has_impl(typify::TypeSpaceImpl::FromStr) {
                    base.ident()
                } else {
                    quote! { String }
                };
                (effective, optional)
            }
            OperationParameterType::RawBody => (quote! { String }, !kind_required),
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

enum StatusPredicate {
    Always,
    Never,
    Expr(TokenStream),
}

impl StatusPredicate {
    fn invalid_condition(self) -> Option<TokenStream> {
        match self {
            StatusPredicate::Always => None,
            StatusPredicate::Never => Some(quote! { true }),
            StatusPredicate::Expr(expr) => Some(quote! { !(#expr) }),
        }
    }
}

fn response_status_guard(
    operation_id: &str,
    items: &[OperationResponse],
    side: ResponseSide,
    is_synth: bool,
    success_items: Option<(&[OperationResponse], bool)>,
) -> TokenStream {
    let invalid_condition = match side {
        ResponseSide::Success => response_status_predicate(items, side, is_synth).invalid_condition(),
        ResponseSide::Error => {
            let error_match = response_status_predicate(items, side, is_synth);
            let (success_items, success_is_synth) = success_items.unwrap_or((&[], false));
            let success_match =
                response_status_predicate(success_items, ResponseSide::Success, success_is_synth);
            match (error_match, success_match) {
                (StatusPredicate::Never, _) => Some(quote! { true }),
                (_, StatusPredicate::Always) => Some(quote! { true }),
                (StatusPredicate::Always, StatusPredicate::Never) => None,
                (StatusPredicate::Always, StatusPredicate::Expr(success)) => {
                    Some(quote! { #success })
                }
                (StatusPredicate::Expr(error), StatusPredicate::Never) => {
                    Some(quote! { !(#error) })
                }
                (StatusPredicate::Expr(error), StatusPredicate::Expr(success)) => {
                    Some(quote! { !(#error) || #success })
                }
            }
        }
    };
    let Some(invalid_condition) = invalid_condition else {
        return quote! {};
    };
    let side_name = match side {
        ResponseSide::Success => "success",
        ResponseSide::Error => "error",
    };
    quote! {
        {
            let __code = __status.as_u16();
            if #invalid_condition {
                return ::progenitor_server::respond::internal(Box::new(
                    ::std::io::Error::new(
                        ::std::io::ErrorKind::Other,
                        ::std::format!(
                            "operation `{}` returned undeclared {} status {}",
                            #operation_id,
                            #side_name,
                            __status,
                        ),
                    ),
                ));
            }
        }
    }
}

/// Predicate, in terms of a generated local `__code: u16`, that mirrors the
/// client-side response classifier for a single response side.
fn response_status_predicate(
    items: &[OperationResponse],
    side: ResponseSide,
    is_synth: bool,
) -> StatusPredicate {
    let mut clauses = Vec::new();
    for item in items {
        let clause = match item.status_code {
            OperationResponseStatus::Code(code) => quote! { __code == #code },
            OperationResponseStatus::Range(range) => {
                let min = range * 100;
                let max = min + 99;
                match side {
                    ResponseSide::Success if !is_synth => quote! { matches!(__code, 200..=299) },
                    _ => quote! { matches!(__code, #min..=#max) },
                }
            }
            OperationResponseStatus::Default => match side {
                // In the non-synth client path, a success-side `default`
                // collapses to the normal 2xx success bucket. In a synthesized
                // success enum, the client emits `_` for the default arm, so it
                // classifies every otherwise-unmatched status as success.
                ResponseSide::Success if is_synth => return StatusPredicate::Always,
                ResponseSide::Success => quote! { matches!(__code, 200..=299) },
                ResponseSide::Error => return StatusPredicate::Always,
            },
        };
        clauses.push(clause);
    }

    let mut clauses = clauses.into_iter();
    let Some(first) = clauses.next() else {
        return StatusPredicate::Never;
    };
    StatusPredicate::Expr(clauses.fold(first, |acc, clause| quote! { #acc || #clause }))
}

fn server_synth_ident(name: &str, side: SynthSide) -> proc_macro2::Ident {
    match side {
        SynthSide::Success => format_ident!("{}", name),
        // Keep `{Op}Error` available as the public `ServerError<E>` alias.
        SynthSide::Error => format_ident!("{}Response", name),
    }
}

fn needs_explicit_synth_status(status: &OperationResponseStatus) -> bool {
    !matches!(status, OperationResponseStatus::Code(_))
}

fn synth_variant_status_arm(
    enum_ident: &proc_macro2::Ident,
    item: &OperationResponse,
) -> TokenStream {
    let variant_ident = format_ident!("{}", synth_variant_name(&item.status_code));
    match item.status_code {
        OperationResponseStatus::Code(code) => {
            let pattern = if matches!(&item.typ, OperationResponseKind::None) {
                quote! { #enum_ident::#variant_ident }
            } else {
                quote! { #enum_ident::#variant_ident(..) }
            };
            quote! {
                #pattern => http::StatusCode::from_u16(#code).unwrap()
            }
        }
        OperationResponseStatus::Range(_) | OperationResponseStatus::Default => {
            quote! {
                #enum_ident::#variant_ident { status, .. } => *status
            }
        }
    }
}

fn synth_variant_pattern(
    enum_ident: &proc_macro2::Ident,
    item: &OperationResponse,
    status_ident: TokenStream,
) -> TokenStream {
    let variant_ident = format_ident!("{}", synth_variant_name(&item.status_code));
    let explicit_status = needs_explicit_synth_status(&item.status_code);
    match (&item.typ, explicit_status) {
        (OperationResponseKind::None, false) => quote! { #enum_ident::#variant_ident },
        (OperationResponseKind::None, true) => {
            quote! { #enum_ident::#variant_ident { status: #status_ident } }
        }
        (_, false) => quote! { #enum_ident::#variant_ident(__body) },
        (_, true) => quote! { #enum_ident::#variant_ident { status: #status_ident, body: __body } },
    }
}

fn synth_variant_status_value(item: &OperationResponse, status_ident: TokenStream) -> TokenStream {
    match item.status_code {
        OperationResponseStatus::Code(code) => {
            quote! { let #status_ident = http::StatusCode::from_u16(#code).unwrap(); }
        }
        OperationResponseStatus::Range(_) | OperationResponseStatus::Default => quote! {},
    }
}

fn synth_variant_status_guard(
    operation_id: &str,
    item: &OperationResponse,
    earlier_items: &[OperationResponse],
    side: ResponseSide,
    status_ident: TokenStream,
) -> TokenStream {
    let earlier_match = status_predicate_for_items(earlier_items);
    let invalid_condition = match item.status_code {
        OperationResponseStatus::Code(code) => {
            let mut shadowing = earlier_items
                .iter()
                .filter(|item| status_covers_code(&item.status_code, code))
                .map(status_predicate_for_item);
            let Some(first) = shadowing.next() else {
                return quote! {};
            };
            shadowing.fold(first, |acc, clause| quote! { #acc || #clause })
        }
        OperationResponseStatus::Range(range) => {
            let min = range * 100;
            let max = min + 99;
            match earlier_match {
                Some(earlier_match) => {
                    quote! { !matches!(__code, #min..=#max) || #earlier_match }
                }
                None => quote! { !matches!(__code, #min..=#max) },
            }
        }
        OperationResponseStatus::Default => match earlier_match {
            Some(earlier_match) => earlier_match,
            None => return quote! {},
        },
    };
    let side_name = match side {
        ResponseSide::Success => "success",
        ResponseSide::Error => "error",
    };
    quote! {
        {
            let __code = #status_ident.as_u16();
            if #invalid_condition {
                return ::progenitor_server::respond::internal(Box::new(
                    ::std::io::Error::new(
                        ::std::io::ErrorKind::Other,
                        ::std::format!(
                            "operation `{}` returned status {} with mismatched {} response variant",
                            #operation_id,
                            #status_ident,
                            #side_name,
                        ),
                    ),
                ));
            }
        }
    }
}

fn status_predicate_for_items(items: &[OperationResponse]) -> Option<TokenStream> {
    let mut clauses = items.iter().map(status_predicate_for_item);
    let first = clauses.next()?;
    Some(clauses.fold(first, |acc, clause| quote! { #acc || #clause }))
}

fn status_predicate_for_item(item: &OperationResponse) -> TokenStream {
    match item.status_code {
        OperationResponseStatus::Code(code) => quote! { __code == #code },
        OperationResponseStatus::Range(range) => {
            let min = range * 100;
            let max = min + 99;
            quote! { matches!(__code, #min..=#max) }
        }
        OperationResponseStatus::Default => quote! { true },
    }
}

fn status_covers_code(status: &OperationResponseStatus, code: u16) -> bool {
    match status {
        OperationResponseStatus::Code(other) => *other == code,
        OperationResponseStatus::Range(range) => {
            let min = range * 100;
            let max = min + 99;
            (min..=max).contains(&code)
        }
        OperationResponseStatus::Default => true,
    }
}

fn synth_success_override_guard(operation_id: &str) -> TokenStream {
    quote! {
        if let Some(__override_status) = __status_override {
            if __override_status != __status {
                return ::progenitor_server::respond::internal(Box::new(
                    ::std::io::Error::new(
                        ::std::io::ErrorKind::Other,
                        ::std::format!(
                            "operation `{}` returned conflicting success status {} for synth variant status {}",
                            #operation_id,
                            __override_status,
                            __status,
                        ),
                    ),
                ));
            }
        }
    }
}

fn synth_error_status_match_guard(operation_id: &str) -> TokenStream {
    quote! {
        if __variant_status != __status {
            return ::progenitor_server::respond::internal(Box::new(
                ::std::io::Error::new(
                    ::std::io::ErrorKind::Other,
                    ::std::format!(
                        "operation `{}` returned conflicting error status {} for synth variant status {}",
                        #operation_id,
                        __status,
                        __variant_status,
                    ),
                ),
            ));
        }
    }
}

/// The lowest concrete 2xx in the success set, else 200.
fn default_success_status(items: &[OperationResponse]) -> u16 {
    items
        .iter()
        .filter_map(|item| match item.status_code {
            OperationResponseStatus::Code(code @ 200..=299) => Some(code),
            _ => None,
        })
        .min()
        .unwrap_or(200)
}

/// The `content-type` for a raw response: the first item's recorded media type,
/// else `application/octet-stream`.
fn raw_content_type(items: &[OperationResponse]) -> String {
    items
        .iter()
        .find_map(|item| item.media_type.clone())
        .unwrap_or_else(|| "application/octet-stream".to_string())
}
