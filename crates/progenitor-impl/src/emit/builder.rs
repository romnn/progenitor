use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::method::{MethodSigBody, make_doc_comment};
use crate::{
    Generator, PreparedIr, Result, TagStyle, ir,
    operation::{
        BodyContentType, DROPSHOT_LIMIT_PARAM, OperationMethod, OperationParameter,
        OperationParameterKind, OperationParameterType,
    },
    util::{Case, sanitize, unique_ident_from},
};

struct BuilderImpl {
    doc: String,
    sig: TokenStream,
    body: TokenStream,
}

struct BuilderParameter {
    name: proc_macro2::Ident,
    typ: TokenStream,
    initial_value: TokenStream,
    finalize: TokenStream,
    implementation: TokenStream,
}

impl Generator {
    fn builder_parameter(
        &self,
        method: &OperationMethod,
        param: &OperationParameter,
        cloneable: &mut bool,
    ) -> Result<BuilderParameter> {
        let name = format_ident!("{}", param.name);
        let (typ, initial_value, finalize, implementation) = match &param.typ {
            OperationParameterType::Type(type_id) => {
                let ty = self.type_space.get_type(type_id)?;
                let builder = ty.builder();
                let type_name = ty.ident();

                let typ = if let (OperationParameterKind::Body(_), Some(builder_name)) =
                    (&param.kind, builder.as_ref())
                {
                    quote! { ::std::result::Result<#builder_name, ::std::string::String> }
                } else if param.optional {
                    quote! { ::std::result::Result<::std::option::Option<#type_name>, ::std::string::String> }
                } else {
                    quote! { ::std::result::Result<#type_name, ::std::string::String> }
                };

                let initial_value =
                    if matches!(param.kind, OperationParameterKind::Body(_)) && builder.is_some() {
                        quote! { Ok(::std::default::Default::default()) }
                    } else if param.optional {
                        quote! { Ok(None) }
                    } else {
                        let error = format!("{} was not initialized", param.name);
                        quote! { Err(#error.to_string()) }
                    };

                let finalize = if builder.is_some() {
                    quote! {
                        .and_then(|v| #type_name::try_from(v)
                            .map_err(|e| e.to_string()))
                    }
                } else {
                    quote! {}
                };

                let implementation = match (builder.as_ref(), param.optional) {
                    (Some(_), true) => unreachable!(),
                    (None, optional) => {
                        let error =
                            format!("conversion to `{}` for {} failed", ty.name(), param.name,);
                        let assign = if optional {
                            quote! { value.try_into().map(Some) }
                        } else {
                            quote! { value.try_into() }
                        };
                        quote! {
                            pub fn #name<V>(mut self, value: V) -> Self
                            where
                                V: std::convert::TryInto<#type_name>,
                            {
                                self.#name = #assign
                                    .map_err(|_| #error.to_string());
                                self
                            }
                        }
                    }
                    (Some(builder_name), false) => {
                        assert_eq!(param.name, "body");
                        let error = format!(
                            "conversion to `{}` for {} failed: {{}}",
                            ty.name(),
                            param.name,
                        );
                        quote! {
                            pub fn body<V>(mut self, value: V) -> Self
                            where
                                V: std::convert::TryInto<#type_name>,
                                <V as std::convert::TryInto<#type_name>>::Error:
                                    std::fmt::Display,
                            {
                                self.body = value.try_into()
                                    .map(From::from)
                                    .map_err(|s| format!(#error, s));
                                self
                            }

                            pub fn body_map<F>(mut self, f: F) -> Self
                            where
                                F: std::ops::FnOnce(#builder_name) -> #builder_name,
                            {
                                self.body = self.body.map(f);
                                self
                            }
                        }
                    }
                };

                (typ, initial_value, finalize, implementation)
            }
            OperationParameterType::RawBody => {
                *cloneable = false;
                let error = format!("{} was not initialized", param.name);
                let implementation = match &param.kind {
                    OperationParameterKind::Body(
                        BodyContentType::OctetStream | BodyContentType::Raw(_),
                    ) => {
                        let conversion_error =
                            format!("conversion to `reqwest::Body` for {} failed", param.name);
                        quote! {
                            pub fn #name<B>(mut self, value: B) -> Self
                            where
                                B: std::convert::TryInto<reqwest::Body>,
                            {
                                self.#name = value.try_into()
                                    .map_err(|_| #conversion_error.to_string());
                                self
                            }
                        }
                    }
                    OperationParameterKind::Body(BodyContentType::Text(_)) => {
                        let conversion_error =
                            format!("conversion to `String` for {} failed", param.name);
                        quote! {
                            pub fn #name<V>(mut self, value: V) -> Self
                            where
                                V: std::convert::TryInto<String>,
                            {
                                self.#name = value
                                    .try_into()
                                    .map_err(|_| #conversion_error.to_string())
                                    .map(|v| v.into());
                                self
                            }
                        }
                    }
                    _ => unreachable!(),
                };
                (
                    quote! { ::std::result::Result<reqwest::Body, ::std::string::String> },
                    quote! { Err(#error.to_string()) },
                    quote! {},
                    implementation,
                )
            }
            OperationParameterType::Multipart(_) => {
                let body_ident = method.multipart_body_ident();
                let error = format!("{} was not initialized", param.name);
                let conversion_error =
                    format!("conversion to `{body_ident}` for {} failed", param.name,);
                (
                    quote! {
                        ::std::result::Result<#body_ident, ::std::string::String>
                    },
                    quote! { Err(#error.to_string()) },
                    quote! {},
                    quote! {
                        pub fn #name<V>(mut self, value: V) -> Self
                        where
                            V: std::convert::TryInto<#body_ident>,
                        {
                            self.#name = value.try_into()
                                .map_err(|_| #conversion_error.to_string());
                            self
                        }
                    },
                )
            }
        };

        Ok(BuilderParameter {
            name,
            typ,
            initial_value,
            finalize,
            implementation,
        })
    }

    /// Create the builder structs along with their impl bodies.
    ///
    /// Builder structs are generally of this form for a mandatory `param_1`
    /// and an optional `param_2`:
    /// ```ignore
    /// struct OperationId<'a> {
    ///     client: &'a super::Client,
    ///     param_1: Result<SomeType, String>,
    ///     param_2: Result<Option<String>, String>,
    /// }
    /// ```
    ///
    /// All parameters are present and all their types are `Result<T, String>`
    /// or `Result<Option<T>, String>` for optional parameters. Each parameter
    /// also has a corresponding method:
    /// ```ignore
    /// impl<'a> OperationId<'a> {
    ///     pub fn param_1<V>(self, value: V)
    ///         where V: std::convert::TryInto<SomeType>
    ///     {
    ///         self.param_1 = value.try_into()
    ///             .map_err(|_| #err_msg.to_string());
    ///         self
    ///     }
    ///     pub fn param_2<V>(self, value: V)
    ///         where V: std::convert::TryInto<SomeType>
    ///     {
    ///         self.param_2 = value.try_into()
    ///             .map(Some)
    ///             .map_err(|_| #err_msg.to_string());
    ///         self
    ///     }
    /// }
    /// ```
    ///
    /// The Client's `operation_id` method simply invokes the builder's new
    /// method, which assigns an error value to mandatory field and a
    /// `Ok(None)` value to optional ones:
    /// ```ignore
    /// impl<'a> OperationId<'a> {
    ///     pub fn new(client: &'a super::Client) -> Self {
    ///         Self {
    ///             client,
    ///             param_1: Err("param_1 was not initialized".to_string()),
    ///             param_2: Ok(None),
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Finally, builders have methods to execute the operation. This simply
    /// resolves each parameter with the ? (`Try` operator).
    /// ```ignore
    /// impl<'a> OperationId<'a> {
    ///     pub fn send(self) -> Result<
    ///         ResponseValue<SuccessType>,
    ///         Error<ErrorType>,
    ///     > {
    ///         let Self {
    ///             client,
    ///             param_1,
    ///             param_2,
    ///         } = self;
    ///     
    ///         let param_1 = param_1.map_err(Error::InvalidRequest)?;
    ///         let param_2 = param_1.map_err(Error::InvalidRequest)?;
    ///
    ///         // ... execute the body (see `method_sig_body`) ...
    ///     }
    /// }
    /// ```
    ///
    /// Finally, paginated interfaces have a `stream()` method which uses the
    /// `send()` method above to fetch each page of results to assemble the
    /// items into a single `impl Stream`.
    /// Generate the per-operation builder struct + `impl` (for the
    /// builder interface style).
    ///
    /// Returns `(extra_types, builder)` analogous to
    /// [`positional_method`] — `extra_types` carries any module-level
    /// items (currently the synthesized response/error sum-type enums)
    /// that must be emitted alongside the builder rather than inside its
    /// `impl` block.
    pub(crate) fn builder_struct(
        &mut self,
        prepared: &PreparedIr,
        method: &OperationMethod,
        tag_style: TagStyle,
    ) -> Result<(TokenStream, TokenStream)> {
        let struct_name = sanitize(&method.operation_id, Case::Pascal);
        let struct_ident = format_ident!("{}", struct_name);

        let mut cloneable = true;
        let parameters = method
            .params
            .iter()
            .map(|param| self.builder_parameter(method, param, &mut cloneable))
            .collect::<Result<Vec<_>>>()?;
        let param_names = parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<Vec<_>>();
        let client_ident = unique_ident_from("client", &param_names);
        let param_types = parameters.iter().map(|parameter| &parameter.typ);
        let param_values = parameters.iter().map(|parameter| &parameter.initial_value);
        let param_finalize = parameters.iter().map(|parameter| &parameter.finalize);
        let param_impls = parameters.iter().map(|parameter| &parameter.implementation);

        let MethodSigBody {
            success,
            error,
            body,
            extra_types,
        } = self.method_sig_body(
            prepared,
            method,
            quote! { super::Client },
            quote! { #client_ident },
        )?;

        let send_doc = format!(
            "Sends a `{}` request to `{}`",
            method.method.as_str().to_ascii_uppercase(),
            method.path,
        );
        let send_impl = quote! {
            #[doc = #send_doc]
            pub async fn send(self) -> Result<
                ResponseValue<#success>,
                Error<#error>,
            > {
                // Destructure the builder for convenience.
                let Self {
                    #client_ident,
                    #( #param_names, )*
                } = self;

                // Extract parameters into variables, returning an error if
                // a value has not been provided or there was a conversion
                // error.
                //
                // TODO we could do something a bit nicer by collecting all
                // errors rather than just reporting the first one.
                #(
                let #param_names =
                    #param_names
                        #param_finalize
                        .map_err(Error::InvalidRequest)?;
                )*

                // Do the work.
                #body
            }
        };

        let stream_impl = method.dropshot_paginated.as_ref().map(|page_data| {
            // We're now using futures.
            self.uses_futures = true;

            let step_params = method.params.iter().filter_map(|param| {
                if param.api_name != DROPSHOT_LIMIT_PARAM
                    && matches!(param.kind, OperationParameterKind::Query { .. })
                {
                    // Query parameters (other than "limit") are None; having
                    // page_token as Some(_), as we will during the loop below,
                    // is mutually exclusive with other query parameters.
                    let name = format_ident!("{}", param.name);
                    Some(quote! {
                        #name: Ok(None)
                    })
                } else {
                    None
                }
            });

            // The item type that we've saved (by picking apart the original
            // function's return type) will be the Item type parameter for the
            // Stream impl we return.
            let item = self.type_space.get_type(&page_data.item).unwrap();
            let item_type = item.ident();

            let stream_doc = format!(
                "Streams `{}` requests to `{}`",
                method.method.as_str().to_ascii_uppercase(),
                method.path,
            );

            quote! {
                #[doc = #stream_doc]
                pub fn stream(self) -> impl futures::Stream<Item = Result<
                    #item_type,
                    Error<#error>,
                >> + Unpin + 'a {
                    use ::futures::StreamExt;
                    use ::futures::TryFutureExt;
                    use ::futures::TryStreamExt;

                    // This is the builder template we'll use for iterative
                    // steps past the first; it has all query params set to
                    // None (the step will fill in page_token).
                    let next = Self {
                        #( #step_params, )*
                        ..self.clone()
                    };

                    self.send()
                        .map_ok(move |page| {
                            let page = page.into_inner();

                            // Create a stream from the first page of items.
                            let first =
                                futures::stream::iter(page.items).map(Ok);

                            // We unfold subsequent pages using page.next_page
                            // as the seed value. Each iteration returns its
                            // items and the new state which is a tuple of the
                            // next page token and the Self template.
                            let rest = futures::stream::try_unfold(
                                (page.next_page, next),
                                |(next_page, next)| async {
                                    if next_page.is_none() {
                                        // The page_token was None so we've
                                        // reached the end.
                                        Ok(None)
                                    } else {
                                        // Get the next page using the next
                                        // template (with query parameters set
                                        // to None), overriding page_token.
                                        Self {
                                            page_token: Ok(next_page),
                                            ..next.clone()
                                        }
                                        .send()
                                        .map_ok(|page| {
                                            let page = page.into_inner();
                                            Some((
                                                futures::stream::iter(
                                                    page.items
                                                ).map(Ok),
                                                (page.next_page, next),
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

        let mut derives = vec![quote! { Debug }];
        if cloneable {
            derives.push(quote! { Clone });
        }

        let derive = quote! {
            #[derive( #( #derives ),* )]
        };

        // Build a reasonable doc comment depending on whether this struct is
        // the output from
        // 1. A Client method
        // 2. An extension trait method
        // 3. Several extension trait methods
        let struct_doc = match (tag_style, method.tags.len(), method.tags.first()) {
            (TagStyle::Merged, _, _) | (TagStyle::Separate, 0, _) => {
                let ty = format!("Client::{}", method.operation_id);
                format!("Builder for [`{ty}`]\n\n[`{ty}`]: super::{ty}")
            }
            (TagStyle::Separate, 1, Some(tag)) => {
                let ty = format!(
                    "Client{}Ext::{}",
                    sanitize(tag, Case::Pascal),
                    method.operation_id
                );
                format!("Builder for [`{ty}`]\n\n[`{ty}`]: super::{ty}")
            }
            (TagStyle::Separate, _, _) => {
                format!(
                    "Builder for `{}` operation\n\nSee {}\n\n{}",
                    method.operation_id,
                    method
                        .tags
                        .iter()
                        .map(|tag| {
                            format!(
                                "[`Client{}Ext::{}`]",
                                sanitize(tag, Case::Pascal),
                                method.operation_id,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                    method
                        .tags
                        .iter()
                        .map(|tag| {
                            let ty = format!(
                                "Client{}Ext::{}",
                                sanitize(tag, Case::Pascal),
                                method.operation_id,
                            );
                            format!("[`{ty}`]: super::{ty}")
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            }
        };

        Ok((
            extra_types,
            quote! {
                #[doc = #struct_doc]
                #derive
                pub struct #struct_ident<'a> {
                    #client_ident: &'a super::Client,
                    #( #param_names: #param_types, )*
                }

                impl<'a> #struct_ident<'a> {
                    pub fn new(client: &'a super::Client) -> Self {
                        Self {
                            #client_ident: client,
                            #( #param_names: #param_values, )*
                        }
                    }

                    #( #param_impls )*
                    #send_impl
                    #stream_impl
                }
            },
        ))
    }

    fn builder_helper(&self, method: &OperationMethod) -> BuilderImpl {
        let operation_id = format_ident!("{}", method.operation_id);
        let struct_name = sanitize(&method.operation_id, Case::Pascal);
        let struct_ident = format_ident!("{}", struct_name);

        let params = method
            .params
            .iter()
            .map(|param| format!("\n    .{}({})", param.name, param.name))
            .collect::<String>();

        let eg = format!(
            "\
            let response = client.{}(){}
    .send()
    .await;",
            method.operation_id, params,
        );

        // Note that it would be nice to have a non-ignored example that could
        // be validated by doc tests, but in order to use the Client we need
        // to import it, and in order to import it we need to know the name of
        // the containing crate... which we can't from this context.
        let doc = format!("{}```ignore\n{}\n```", make_doc_comment(method), eg);

        let sig = quote! {
            fn #operation_id(&self) -> builder:: #struct_ident <'_>
        };

        let body = quote! {
            builder:: #struct_ident ::new(self)
        };
        BuilderImpl { doc, sig, body }
    }

    /// Generates a pair of `TokenStreams`.
    ///
    /// The first includes all the operation code; impl Client for operations
    /// with no tags and code of this form for each tag:
    ///
    /// ```ignore
    /// pub trait ClientTagExt {
    ///     ...
    /// }
    ///
    /// impl ClientTagExt for Client {
    ///     ...
    /// }
    /// ```
    ///
    /// The second is the code for the prelude for each tag extension trait:
    ///
    /// ```ignore
    /// pub use super::ClientTagExt;
    /// ```
    pub(crate) fn builder_tags(
        &self,
        methods: &[OperationMethod],
        tag_info: &BTreeMap<&String, &ir::Tag>,
    ) -> (TokenStream, TokenStream) {
        let mut base = Vec::new();
        let mut ext = BTreeMap::new();

        methods.iter().for_each(|method| {
            let BuilderImpl { doc, sig, body } = self.builder_helper(method);

            if method.tags.is_empty() {
                let impl_body = quote! {
                    #[doc = #doc]
                    pub #sig {
                        #body
                    }
                };
                base.push(impl_body);
            } else {
                let trait_sig = quote! {
                    #[doc = #doc]
                    #sig;
                };

                let impl_body = quote! {
                    #sig {
                        #body
                    }
                };
                method.tags.iter().for_each(|tag| {
                    ext.entry(tag.clone())
                        .or_insert_with(Vec::new)
                        .push((trait_sig.clone(), impl_body.clone()));
                });
            }
        });

        let base_impl = (!base.is_empty()).then(|| {
            quote! {
                impl Client {
                    #(#base)*
                }
            }
        });

        let (ext_impl, ext_use): (Vec<_>, Vec<_>) = ext
            .into_iter()
            .map(|(tag, trait_methods)| {
                let desc = tag_info
                    .get(&tag)
                    .and_then(|tag| tag.description.as_ref())
                    .map(|d| quote! { #[doc = #d] });
                let tr = format_ident!("Client{}Ext", sanitize(&tag, Case::Pascal));
                let (trait_methods, trait_impls): (Vec<TokenStream>, Vec<TokenStream>) =
                    trait_methods.into_iter().unzip();
                (
                    quote! {
                        #desc
                        pub trait #tr {
                            #(#trait_methods)*
                        }

                        impl #tr for Client {
                            #(#trait_impls)*
                        }
                    },
                    tr,
                )
            })
            .unzip();

        (
            quote! {
                #base_impl

                #(#ext_impl)*
            },
            quote! {
                #(pub use super::#ext_use;)*
            },
        )
    }

    pub(crate) fn builder_impl(&self, method: &OperationMethod) -> TokenStream {
        let BuilderImpl { doc, sig, body } = self.builder_helper(method);

        let impl_body = quote! {
            #[doc = #doc]
            pub #sig {
                #body
            }
        };

        impl_body
    }
}
