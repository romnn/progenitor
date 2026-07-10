use super::{
    BodyContentType, DROPSHOT_LIMIT_PARAM, DROPSHOT_PAGE_TOKEN_PARAM, Generator, MethodSigBody,
    OperationMethod, OperationParameterKind, OperationParameterType, PreparedIr, Result,
    TokenStream, format_ident, make_doc_comment, make_stream_doc_comment, quote,
};
use quote::ToTokens;

impl Generator {
    /// Generate the `impl`-block body for one positional-style method.
    ///
    /// Returns `(extra_types, impl_body)` where `extra_types` contains any
    /// items (today: per-operation synthesized response/error sum-type
    /// enums) that must live *outside* the surrounding `impl Client {}`
    /// block — a `pub enum` is rejected by the parser inside an impl. The
    /// caller is expected to emit the extras at module level alongside
    /// the impl.
    pub(crate) fn positional_method(
        &mut self,
        prepared: &PreparedIr,
        method: &OperationMethod,
        has_inner: bool,
    ) -> Result<(TokenStream, TokenStream)> {
        let operation_id = format_ident!("{}", method.operation_id);

        // Render each parameter as it will appear in the method signature.
        let params = method
            .params
            .iter()
            .map(|param| {
                let name = format_ident!("{}", param.name);
                let typ = match (&param.typ, param.optional) {
                    (OperationParameterType::Type(type_id), false) => self
                        .type_space
                        .get_type(type_id)
                        .unwrap()
                        .parameter_ident_with_lifetime("a"),
                    (OperationParameterType::Type(type_id), true) => {
                        let t = self
                            .type_space
                            .get_type(type_id)
                            .unwrap()
                            .parameter_ident_with_lifetime("a");
                        quote! { ::std::option::Option<#t> }
                    }
                    (OperationParameterType::RawBody, false) => match &param.kind {
                        OperationParameterKind::Body(
                            BodyContentType::OctetStream | BodyContentType::Raw(_),
                        ) => {
                            quote! { B }
                        }
                        OperationParameterKind::Body(BodyContentType::Text(_)) => {
                            quote! { ::std::string::String }
                        }
                        _ => unreachable!(),
                    },
                    (OperationParameterType::RawBody, true) => unreachable!(),
                };
                quote! {
                    #name: #typ
                }
            })
            .collect::<Vec<_>>();

        let raw_body_param = method.params.iter().any(|param| {
            param.typ == OperationParameterType::RawBody
                && matches!(
                    &param.kind,
                    OperationParameterKind::Body(
                        BodyContentType::OctetStream | BodyContentType::Raw(_)
                    )
                )
        });

        let bounds = if raw_body_param {
            quote! { <'a, B: Into<reqwest::Body> > }
        } else {
            quote! { <'a> }
        };

        let doc_comment = make_doc_comment(method);

        let MethodSigBody {
            success: success_type,
            error: error_type,
            body,
            extra_types,
        } = self.method_sig_body(
            prepared,
            method,
            quote! { Self },
            quote! { self },
            has_inner,
        )?;

        let method_impl = quote! {
            #[doc = #doc_comment]
            pub async fn #operation_id #bounds (
                &'a self,
                #(#params),*
            ) -> Result<
                ResponseValue<#success_type>,
                Error<#error_type>,
            > {
                #body
            }
        };

        let stream_impl = method.dropshot_paginated.as_ref().map(|page_data| {
            // We're now using futures.
            self.uses_futures = true;

            let stream_id = format_ident!("{}_stream", method.operation_id);

            // The parameters are the same as those to the paged method, but
            // without "page_token"
            let stream_params = method
                .params
                .iter()
                .zip(params)
                .filter_map(|(param, stream)| {
                    if param.name == DROPSHOT_PAGE_TOKEN_PARAM {
                        None
                    } else {
                        Some(stream)
                    }
                });

            // The values passed to get the first page are the inputs to the
            // stream method with "None" for the page_token.
            let first_params = method.params.iter().map(|param| {
                if param.api_name == DROPSHOT_PAGE_TOKEN_PARAM {
                    // The page_token is None when getting the first page.
                    quote! { None }
                } else {
                    // All other parameters are passed through directly.
                    format_ident!("{}", param.name).to_token_stream()
                }
            });

            // The values passed to get subsequent pages are...
            // - the state variable for the page_token
            // - None for all other query parameters
            // - The initial inputs for non-query parameters
            let step_params = method.params.iter().map(|param| {
                if param.api_name == DROPSHOT_PAGE_TOKEN_PARAM {
                    quote! { state.as_deref() }
                } else if param.api_name != DROPSHOT_LIMIT_PARAM
                    && matches!(param.kind, OperationParameterKind::Query { .. })
                {
                    // Query parameters (other than "page_token" and "limit")
                    // are None; having page_token as Some(_) is mutually
                    // exclusive with other query parameters.
                    quote! { None }
                } else {
                    // Non-query parameters are passed in; this is necessary
                    // e.g. to specify the right path. (We don't really expect
                    // to see a body parameter here, but we pass it through
                    // regardless.)
                    format_ident!("{}", param.name).to_token_stream()
                }
            });

            // The item type that we've saved (by picking apart the original
            // function's return type) will be the Item type parameter for the
            // Stream type we return.
            let item = self.type_space.get_type(&page_data.item).unwrap();
            let item_type = item.ident();

            let doc_comment = make_stream_doc_comment(method);

            quote! {
                #[doc = #doc_comment]
                pub fn #stream_id #bounds (
                    &'a self,
                    #(#stream_params),*
                ) -> impl futures::Stream<Item = Result<
                    #item_type,
                    Error<#error_type>,
                >> + Unpin + 'a {
                    use futures::StreamExt;
                    use futures::TryFutureExt;
                    use futures::TryStreamExt;

                    // Execute the operation with the basic parameters
                    // (omitting page_token) to get the first page.
                    self.#operation_id( #(#first_params,)* )
                        .map_ok(move |page| {
                            let page = page.into_inner();

                            // Create a stream from the items of the first page.
                            let first =
                                futures::stream::iter(page.items).map(Ok);

                            // We unfold subsequent pages using page.next_page
                            // as the seed value. Each iteration returns its
                            // items and the next page token.
                            let rest = futures::stream::try_unfold(
                                page.next_page,
                                move |state| async move {
                                    if state.is_none() {
                                        // The page_token was None so we've
                                        // reached the end.
                                        Ok(None)
                                    } else {
                                        // Get the next page; here we set all
                                        // query parameters to None (except for
                                        // the page_token), and all other
                                        // parameters as specified at the start
                                        // of this method.
                                        self.#operation_id(
                                            #(#step_params,)*
                                        )
                                        .map_ok(|page| {
                                            let page = page.into_inner();
                                            Some((
                                                futures::stream::iter(
                                                    page.items
                                                ).map(Ok),
                                                page.next_page,
                                            ))
                                        })
                                        .await
                                    }
                                },
                            )
                            .try_flatten();

                            first.chain(rest)
                        })
                        .try_flatten_stream()
                        .boxed()
                }
            }
        });

        let all = quote! {
            #method_impl
            #stream_impl
        };

        Ok((extra_types, all))
    }
}
