// Copyright 2025 Oxide Computer Company

//! Core implementation for the progenitor `OpenAPI` client generator.

#![deny(missing_docs)]

use std::collections::{BTreeMap, HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use thiserror::Error;
use typify::{TypeId, TypeSpace, TypeSpaceSettings};

pub use crate::ir::OpenApiDocument;
pub use crate::openapi::ParseOpenApiError;
pub use crate::openapi::parse_openapi_str;
pub use crate::openapi::parse_openapi_value;
pub use typify::CrateVers;
pub use typify::TypeSpaceImpl as TypeImpl;
pub use typify::TypeSpacePatch as TypePatch;
pub use typify::UnknownPolicy;

mod cli;
mod httpmock;
mod ir;
mod method;
mod openapi;
mod operation;
mod server;
mod template;
mod util;

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("unexpected value type {0}: {1}")]
    BadValue(String, serde_json::Value),
    #[error("type error {0}")]
    TypeError(#[from] typify::Error),
    #[error("unexpected or unhandled format in the OpenAPI document {0}")]
    UnexpectedFormat(String),
    #[error("invalid operation path {0}")]
    InvalidPath(String),
    #[error("invalid dropshot extension use: {0}")]
    InvalidExtension(String),
    #[error("internal error {0}")]
    InternalError(String),
}

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, Error>;

/// `OpenAPI` generator.
pub struct Generator {
    type_space: TypeSpace,
    settings: GenerationSettings,
    uses_futures: bool,
    uses_websockets: bool,
    /// Component-schema name → `TypeId` of the corresponding typify type
    /// after `add_ref_types`. Needed alongside `schema_supertypes` so
    /// `extract_responses` can resolve a common-ancestor schema name back
    /// to a usable `TypeId` when rewriting response items.
    schema_type_ids: BTreeMap<String, TypeId>,
    /// Component schemas as schemars objects, retained after `generate_tokens`
    /// so callers can inspect metadata (e.g., examples) without a second parse.
    component_schemas: indexmap::IndexMap<String, schemars::schema::Schema>,
    diagnostics: Vec<String>,
}

/// The shared generation IR produced once per spec by [`Generator::prepare`].
///
/// Owned so it can outlive the `&mut self` token-generation calls (which mutate
/// generator state such as `uses_futures`) without holding a borrow of `self`.
pub(crate) struct PreparedIr {
    /// Operation methods with operation IDs already deduped.
    pub raw_methods: Vec<operation::OperationMethod>,
    pub schema_supertypes: BTreeMap<String, String>,
    pub schema_type_ids: BTreeMap<String, TypeId>,
    #[expect(
        dead_code,
        reason = "retained with the prepared schema maps for backend metadata consumers"
    )]
    pub component_schemas: indexmap::IndexMap<String, schemars::schema::Schema>,
}

/// Settings for [Generator].
#[derive(Default, Clone)]
pub struct GenerationSettings {
    interface: InterfaceStyle,
    tag: TagStyle,
    /// Emit a server-stub module (service trait + axum adapter) alongside the
    /// client. Default `false`.
    generate_server: bool,
    inner_type: Option<TokenStream>,
    pre_hook: Option<TokenStream>,
    pre_hook_async: Option<TokenStream>,
    post_hook: Option<TokenStream>,
    post_hook_async: Option<TokenStream>,
    extra_derives: Vec<String>,
    extra_cli_bounds: Vec<String>,

    map_type: Option<String>,
    unknown_crates: UnknownPolicy,
    crates: BTreeMap<String, CrateSpec>,

    patch: HashMap<String, TypePatch>,
    replace: HashMap<String, (String, Vec<TypeImpl>)>,
    convert: Vec<(schemars::schema::SchemaObject, String, Vec<TypeImpl>)>,
    timeout: Option<u64>,
}

#[derive(Debug, Clone)]
struct CrateSpec {
    version: CrateVers,
    rename: Option<String>,
}

/// Style of generated client.
#[derive(Clone, Default, Deserialize, PartialEq, Eq)]
pub enum InterfaceStyle {
    /// Use positional style.
    #[default]
    Positional,
    /// Use builder style.
    Builder,
}

/// Style for using the `OpenAPI` tags when generating names in the client.
#[derive(Clone, Default, Deserialize)]
pub enum TagStyle {
    /// Merge tags to create names in the generated client.
    #[default]
    Merged,
    /// Use each tag name to create separate names in the generated client.
    Separate,
}

impl GenerationSettings {
    /// Create new generator settings with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the [`InterfaceStyle`].
    pub fn with_interface(&mut self, interface: InterfaceStyle) -> &mut Self {
        self.interface = interface;
        self
    }

    /// Set the [`TagStyle`].
    pub fn with_tag(&mut self, tag: TagStyle) -> &mut Self {
        self.tag = tag;
        self
    }

    /// Enable generation of the server-stub module (default `false`).
    ///
    /// When enabled, [`Generator::generate_tokens`] appends a `pub mod server`
    /// containing the service trait and an axum adapter. The generated code
    /// references the `progenitor-server` runtime crate, which the consumer must
    /// add to its own dependencies (it is never pulled in by the generator).
    pub fn with_server(&mut self, generate_server: bool) -> &mut Self {
        self.generate_server = generate_server;
        self
    }

    /// Client inner type available to pre and post hooks.
    pub fn with_inner_type(&mut self, inner_type: TokenStream) -> &mut Self {
        self.inner_type = Some(inner_type);
        self
    }

    /// Hook invoked before issuing the HTTP request.
    pub fn with_pre_hook(&mut self, pre_hook: TokenStream) -> &mut Self {
        self.pre_hook = Some(pre_hook);
        self
    }

    /// Hook invoked before issuing the HTTP request.
    pub fn with_pre_hook_async(&mut self, pre_hook: TokenStream) -> &mut Self {
        self.pre_hook_async = Some(pre_hook);
        self
    }

    /// Hook invoked prior to receiving the HTTP response.
    pub fn with_post_hook(&mut self, post_hook: TokenStream) -> &mut Self {
        self.post_hook = Some(post_hook);
        self
    }

    /// Hook invoked prior to receiving the HTTP response.
    pub fn with_post_hook_async(&mut self, post_hook: TokenStream) -> &mut Self {
        self.post_hook_async = Some(post_hook);
        self
    }

    /// Additional derive macros applied to generated types.
    pub fn with_derive(&mut self, derive: impl ToString) -> &mut Self {
        self.extra_derives.push(derive.to_string());
        self
    }

    /// Additional trait bounds applied to `CliConfig` methods.
    pub fn with_cli_bounds(&mut self, derive: impl ToString) -> &mut Self {
        self.extra_cli_bounds.push(derive.to_string());
        self
    }

    /// Modify a type with the given name.
    /// See [`typify::TypeSpaceSettings::with_patch`].
    pub fn with_patch<S: AsRef<str>>(&mut self, type_name: S, patch: &TypePatch) -> &mut Self {
        self.patch
            .insert(type_name.as_ref().to_string(), patch.clone());
        self
    }

    /// Replace a referenced type with a named type.
    /// See [`typify::TypeSpaceSettings::with_replacement`].
    pub fn with_replacement<TS: ToString, RS: ToString, I: Iterator<Item = TypeImpl>>(
        &mut self,
        type_name: TS,
        replace_name: RS,
        impls: I,
    ) -> &mut Self {
        self.replace.insert(
            type_name.to_string(),
            (replace_name.to_string(), impls.collect()),
        );
        self
    }

    /// Replace a given schema with a named type.
    /// See [`typify::TypeSpaceSettings::with_conversion`].
    pub fn with_conversion<S: ToString, I: Iterator<Item = TypeImpl>>(
        &mut self,
        schema: schemars::schema::SchemaObject,
        type_name: S,
        impls: I,
    ) -> &mut Self {
        self.convert
            .push((schema, type_name.to_string(), impls.collect()));
        self
    }

    /// Policy regarding crates referenced by the schema extension
    /// `x-rust-type` not explicitly specified via [`Self::with_crate`].
    /// See [`typify::TypeSpaceSettings::with_unknown_crates`].
    pub fn with_unknown_crates(&mut self, policy: UnknownPolicy) -> &mut Self {
        self.unknown_crates = policy;
        self
    }

    /// Explicitly named crates whose types may be used during generation
    /// rather than generating new types based on their schemas (base on the
    /// presence of the x-rust-type extension).
    /// See [`typify::TypeSpaceSettings::with_crate`].
    pub fn with_crate<S1: ToString>(
        &mut self,
        crate_name: S1,
        version: CrateVers,
        rename: Option<&String>,
    ) -> &mut Self {
        self.crates.insert(
            crate_name.to_string(),
            CrateSpec {
                version,
                rename: rename.cloned(),
            },
        );
        self
    }

    /// Set the type used for key-value maps. Common examples:
    /// - [`std::collections::HashMap`] - **Default**
    /// - [`std::collections::BTreeMap`]
    /// - [`indexmap::IndexMap`]
    ///
    /// The requiremnets for a map type can be found in the
    /// [`typify::TypeSpaceSettings::with_map_type`] documentation.
    pub fn with_map_type<MT: ToString>(&mut self, map_type: MT) -> &mut Self {
        self.map_type = Some(map_type.to_string());
        self
    }

    /// Set the underlying reqwest client's timeout
    pub fn with_timeout(&mut self, timeout: u64) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            type_space: TypeSpace::new(TypeSpaceSettings::default().with_type_mod("types")),
            settings: Default::default(),
            uses_futures: Default::default(),
            uses_websockets: Default::default(),
            schema_type_ids: Default::default(),
            component_schemas: Default::default(),
            diagnostics: Default::default(),
        }
    }
}

impl Generator {
    /// Create a new generator with default values.
    #[must_use]
    pub fn new(settings: &GenerationSettings) -> Self {
        let mut type_settings = TypeSpaceSettings::default();
        type_settings
            .with_type_mod("types")
            .with_struct_builder(settings.interface == InterfaceStyle::Builder);
        settings.extra_derives.iter().for_each(|derive| {
            let _ = type_settings.with_derive(derive.clone());
        });

        // Control use of crates found in x-rust-type extension
        type_settings.with_unknown_crates(settings.unknown_crates);
        settings
            .crates
            .iter()
            .for_each(|(crate_name, CrateSpec { version, rename })| {
                type_settings.with_crate(crate_name, version.clone(), rename.as_ref());
            });

        // Adjust generation by type, name, or schema.
        settings.patch.iter().for_each(|(type_name, patch)| {
            type_settings.with_patch(type_name, patch);
        });
        settings
            .replace
            .iter()
            .for_each(|(type_name, (replace_name, impls))| {
                type_settings.with_replacement(type_name, replace_name, impls.iter().copied());
            });
        settings
            .convert
            .iter()
            .for_each(|(schema, type_name, impls)| {
                type_settings.with_conversion(schema.clone(), type_name, impls.iter().copied());
            });

        // Set the map type if specified.
        if let Some(map_type) = &settings.map_type {
            type_settings.with_map_type(map_type.clone());
        }

        Self {
            type_space: TypeSpace::new(&type_settings),
            settings: settings.clone(),
            uses_futures: false,
            uses_websockets: false,
            schema_type_ids: Default::default(),
            component_schemas: Default::default(),
            diagnostics: Default::default(),
        }
    }

    /// Generation diagnostics that could not be represented in generated code.
    #[must_use]
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    /// Resolve each `components.schemas` entry to its `TypeId` by handing
    /// typify a `$ref` to the named component — `add_ref_types` already
    /// populated typify's internal `ref_to_id` map, so passing the same
    /// `$ref` back returns the cached id rather than synthesising a new
    /// type. The resulting map is the inverse of typify's name → id mapping
    /// that progenitor needs (typify doesn't expose it directly).
    fn build_schema_type_id_map(
        &mut self,
        schemas: &indexmap::IndexMap<String, schemars::schema::Schema>,
    ) -> Result<BTreeMap<String, TypeId>> {
        let mut map = BTreeMap::new();
        for name in schemas.keys() {
            let ref_schema: schemars::schema::Schema =
                schemars::schema::SchemaObject::new_ref(format!("#/components/schemas/{name}"))
                    .into();
            let type_id = self.type_space.add_type_with_name(&ref_schema, None)?;
            map.insert(name.clone(), type_id);
        }
        Ok(map)
    }

    /// Emit the generated client as formatted Rust source. This is the
    /// canonical text form: token-stream `to_string` puts the whole
    /// client on one line (a large spec yields a 100 MB single-line
    /// file whose rustc diagnostics each embed the entire line), so
    /// every consumer that writes source to disk should go through
    /// here.
    pub fn generate_text(&mut self, spec: &OpenApiDocument) -> Result<String> {
        let tokens = self.generate_tokens(spec)?;
        let file = syn::parse2::<syn::File>(tokens)
            .map_err(|e| Error::InternalError(format!("generated code does not parse: {e}")))?;
        Ok(prettyplease::unparse(&file))
    }

    /// Returns `(schema_name, rust_ident, examples)` for every component schema
    /// that maps to a named Rust type and carries at least one example value.
    ///
    /// Must be called after [`generate_text`] or [`generate_tokens`]; returns
    /// an empty vec if called before generation.
    #[must_use]
    pub fn example_schemas(&self) -> Vec<(String, String, Vec<serde_json::Value>)> {
        self.schema_type_ids
            .iter()
            .filter_map(|(schema_name, type_id)| {
                let t = self.type_space.get_type(type_id).ok()?;
                let ident = t.named_ident()?;
                let schema = self.component_schemas.get(schema_name)?;
                let examples = match schema {
                    schemars::schema::Schema::Object(obj) => obj
                        .metadata
                        .as_deref()
                        .map(|m| m.examples.clone())
                        .unwrap_or_default(),
                    _ => Vec::new(),
                };
                if examples.is_empty() {
                    return None;
                }
                Some((schema_name.clone(), ident, examples))
            })
            .collect()
    }

    /// Prepare the shared generation IR for a spec: register component schemas
    /// in the type space, build the supertype / schema-`TypeId` maps that
    /// [`Generator::extract_responses`](crate::method) relies on, retain the
    /// component schemas, and produce the per-operation methods with deduped
    /// operation IDs.
    ///
    /// Returns an **owned** [`PreparedIr`] (not a borrow of `self`) so callers
    /// can pass `&prepared.raw_methods` into the `&mut self` token-generation
    /// methods without holding a borrow of `self`. Shared by `generate_tokens`
    /// and the standalone `server` entry point so the client and server agree on
    /// types and method names; safe to call more than once on the same generator
    /// (`add_ref_types` is idempotent for already-registered names, which the
    /// existing `generate_text`-then-`httpmock` flow already relies on).
    pub(crate) fn prepare(&mut self, spec: &OpenApiDocument) -> Result<PreparedIr> {
        let document = &spec.0;

        self.type_space.add_ref_types(
            document
                .schemas
                .iter()
                .map(|(name, schema)| (name.clone(), schema.clone())),
        )?;

        let schema_supertypes = crate::ir::build_schema_supertype_map(&document.schemas);
        let schema_type_ids = self.build_schema_type_id_map(&document.schemas)?;
        let component_schemas = document.schemas.clone();
        self.schema_type_ids = schema_type_ids.clone();
        self.component_schemas = component_schemas.clone();

        let mut raw_methods = document
            .operations
            .iter()
            .map(|operation| self.process_operation(operation, &document.schemas))
            .collect::<Result<Vec<_>>>()?;

        // Specs sometimes assign the same operationId to multiple paths.
        // Deduplicate by appending _2, _3, … to later occurrences so the
        // generated Rust methods don't collide.
        {
            let mut taken = HashSet::new();
            for method in &mut raw_methods {
                let base = method.operation_id.clone();
                if taken.insert(base.clone()) {
                    continue;
                }

                let mut suffix = 2;
                loop {
                    let candidate = format!("{base}_{suffix}");
                    if taken.insert(candidate.clone()) {
                        method.operation_id = candidate;
                        break;
                    }
                    suffix += 1;
                }
            }
        }

        Ok(PreparedIr {
            raw_methods,
            schema_supertypes,
            schema_type_ids,
            component_schemas,
        })
    }

    /// Emit a [`TokenStream`] containing the generated client code.
    pub fn generate_tokens(&mut self, spec: &OpenApiDocument) -> Result<TokenStream> {
        let document = &spec.0;

        let prepared = self.prepare(spec)?;
        let raw_methods = &prepared.raw_methods;

        let operation_code = match (&self.settings.interface, &self.settings.tag) {
            (InterfaceStyle::Positional, TagStyle::Merged) => self
                .generate_tokens_positional_merged(
                    &prepared,
                    raw_methods,
                    self.settings.inner_type.is_some(),
                ),
            (InterfaceStyle::Positional, TagStyle::Separate) => {
                return Err(Error::UnexpectedFormat(
                    "positional arguments with separate tags are currently unsupported".to_string(),
                ));
            }
            (InterfaceStyle::Builder, TagStyle::Merged) => self.generate_tokens_builder_merged(
                &prepared,
                raw_methods,
                self.settings.inner_type.is_some(),
            ),
            (InterfaceStyle::Builder, TagStyle::Separate) => {
                let tag_info = document
                    .tags
                    .iter()
                    .map(|tag| (&tag.name, tag))
                    .collect::<BTreeMap<_, _>>();
                self.generate_tokens_builder_separate(
                    &prepared,
                    raw_methods,
                    tag_info,
                    self.settings.inner_type.is_some(),
                )
            }
        }?;

        let types = self.type_space.to_stream();

        let (inner_type, inner_fn_value) = match self.settings.inner_type.as_ref() {
            Some(inner_type) => (inner_type.clone(), quote! { &self.inner }),
            None => (quote! { () }, quote! { &() }),
        };

        let inner_property = self.settings.inner_type.as_ref().map(|inner| {
            quote! {
                pub (crate) inner: #inner,
            }
        });
        let inner_parameter = self.settings.inner_type.as_ref().map(|inner| {
            quote! {
                inner: #inner,
            }
        });
        let inner_value = self.settings.inner_type.as_ref().map(|_| {
            quote! {
                inner
            }
        });
        let client_timeout = self.settings.timeout.unwrap_or(15);

        let client_docstring = {
            let mut s = format!("Client for {}", document.info.title);

            if let Some(ss) = &document.info.description {
                s.push_str("\n\n");
                // API-level descriptions are often long markdown intros
                // whose bare code fences rustdoc would run as doctests.
                s.push_str(&util::neutralize_doc_fences(ss));
            }
            if let Some(ss) = &document.info.terms_of_service {
                s.push_str("\n\n");
                s.push_str(ss);
            }

            s.push_str(&format!("\n\nVersion: {}", &document.info.version));

            s
        };

        let version_str = &document.info.version;

        // When server generation is enabled, emit the server-stub module body
        // (trait + axum adapter) and wrap it once in `pub mod server`. When
        // disabled this is an empty `TokenStream`, which contributes zero tokens
        // to the parsed `syn::File`, keeping the formatted client output
        // byte-identical to the flag-off case. The body references the SDK's own
        // types via `crate::types` (this is the in-crate path).
        let server_module = if self.settings.generate_server {
            let server_body = self.server_body(&prepared, document, "crate")?;
            quote! {
                /// Server-stub module: the service trait and its axum adapter.
                ///
                /// Implement the trait, then mount the `{Api}Server` via the
                /// `progenitor-server` runtime (or `into_router()` to compose it
                /// into your own axum app).
                #[allow(clippy::all)]
                pub mod server {
                    #server_body
                }
            }
        } else {
            quote! {}
        };

        // The allow(unused_imports) on the `pub use` is necessary with Rust
        // 1.76+, in case the generated file is not at the top level of the
        // crate.

        let file = quote! {
            // Re-export types that are used by the public interface of Client.
            #[allow(unused_imports)]
            pub use progenitor_client::{
                ByteStream,
                ClientInfo,
                Error,
                ResponseValue,
            };
            #[allow(unused_imports)]
            use progenitor_client::{
                encode_path,
                ClientHooks,
                OperationInfo,
                RequestBuilderExt,
            };

            /// Types used as operation parameters and responses.
            #[allow(clippy::all)]
            pub mod types {
                #types
            }

            #[derive(Clone, Debug)]
            #[doc = #client_docstring]
            pub struct Client {
                pub(crate) baseurl: String,
                pub(crate) client: reqwest::Client,
                #inner_property
            }

            impl Client {
                /// Create a new client.
                ///
                /// `baseurl` is the base URL provided to the internal
                /// `reqwest::Client`, and should include a scheme and hostname,
                /// as well as port and a path stem if applicable.
                pub fn new(
                    baseurl: &str,
                    #inner_parameter
                ) -> Self {
                    #[cfg(not(target_arch = "wasm32"))]
                    let client = {
                        let dur = ::std::time::Duration::from_secs(#client_timeout);

                        reqwest::ClientBuilder::new()
                            .connect_timeout(dur)
                            .timeout(dur)
                    };

                    #[cfg(target_arch = "wasm32")]
                    let client = reqwest::ClientBuilder::new();

                    Self::new_with_client(baseurl, client.build().unwrap(), #inner_value)
                }

                /// Construct a new client with an existing `reqwest::Client`,
                /// allowing more control over its configuration.
                ///
                /// `baseurl` is the base URL provided to the internal
                /// `reqwest::Client`, and should include a scheme and hostname,
                /// as well as port and a path stem if applicable.
                pub fn new_with_client(
                    baseurl: &str,
                    client: reqwest::Client,
                    #inner_parameter
                ) -> Self {
                    Self {
                        baseurl: baseurl.to_string(),
                        client,
                        #inner_value
                    }
                }
            }

            impl ClientInfo<#inner_type> for Client {
                fn api_version() -> &'static str {
                    #version_str
                }

                fn baseurl(&self) -> &str {
                    self.baseurl.as_str()
                }

                fn client(&self) -> &reqwest::Client {
                    &self.client
                }

                fn inner(&self) -> &#inner_type {
                    #inner_fn_value
                }
            }

            impl ClientHooks<#inner_type> for &Client {}

            #operation_code

            #server_module
        };

        Ok(file)
    }

    fn generate_tokens_positional_merged(
        &mut self,
        prepared: &PreparedIr,
        input_methods: &[operation::OperationMethod],
        has_inner: bool,
    ) -> Result<TokenStream> {
        let pairs = input_methods
            .iter()
            .map(|method| self.positional_method(prepared, method, has_inner))
            .collect::<Result<Vec<_>>>()?;
        let (extra_types, methods): (Vec<TokenStream>, Vec<TokenStream>) =
            pairs.into_iter().unzip();

        // The allow(unused_imports) on the `pub use` is necessary with Rust
        // 1.76+, in case the generated file is not at the top level of the
        // crate.

        let out = quote! {
            // Per-operation synthesised response/error enums must live at
            // module level (a `pub enum` inside `impl Client {}` is rejected
            // by the parser), so emit them ahead of the impl.
            #(#extra_types)*

            #[allow(clippy::all)]
            impl Client {
                #(#methods)*
            }

            /// Items consumers will typically use such as the Client.
            pub mod prelude {
                #[allow(unused_imports)]
                pub use super::Client;
            }
        };
        Ok(out)
    }

    fn generate_tokens_builder_merged(
        &mut self,
        prepared: &PreparedIr,
        input_methods: &[operation::OperationMethod],
        has_inner: bool,
    ) -> Result<TokenStream> {
        let pairs = input_methods
            .iter()
            .map(|method| self.builder_struct(prepared, method, TagStyle::Merged, has_inner))
            .collect::<Result<Vec<_>>>()?;
        let (builder_extra_types, builder_struct): (Vec<TokenStream>, Vec<TokenStream>) =
            pairs.into_iter().unzip();

        let builder_methods = input_methods
            .iter()
            .map(|method| self.builder_impl(method))
            .collect::<Vec<_>>();

        let out = quote! {
            impl Client {
                #(#builder_methods)*
            }

            /// Types for composing operation parameters.
            #[allow(clippy::all)]
            pub mod builder {
                use super::types;
                #[allow(unused_imports)]
                use super::{
                    encode_path,
                    ByteStream,
                    ClientInfo,
                    ClientHooks,
                    Error,
                    OperationInfo,
                    RequestBuilderExt,
                    ResponseValue,
                };

                // Synthesised response/error enums live alongside the
                // builders so the `impl<'_> Builder<'_> { send }` arms can
                // name them by short identifier.
                #(#builder_extra_types)*

                #(#builder_struct)*
            }

            /// Items consumers will typically use such as the Client.
            pub mod prelude {
                pub use self::super::Client;
            }
        };

        Ok(out)
    }

    fn generate_tokens_builder_separate(
        &mut self,
        prepared: &PreparedIr,
        input_methods: &[operation::OperationMethod],
        tag_info: BTreeMap<&String, &ir::Tag>,
        has_inner: bool,
    ) -> Result<TokenStream> {
        let pairs = input_methods
            .iter()
            .map(|method| self.builder_struct(prepared, method, TagStyle::Separate, has_inner))
            .collect::<Result<Vec<_>>>()?;
        let (builder_extra_types, builder_struct): (Vec<TokenStream>, Vec<TokenStream>) =
            pairs.into_iter().unzip();

        let (traits_and_impls, trait_preludes) = self.builder_tags(input_methods, &tag_info);

        // The allow(unused_imports) on the `pub use` is necessary with Rust
        // 1.76+, in case the generated file is not at the top level of the
        // crate.

        let out = quote! {
            #traits_and_impls

            /// Types for composing operation parameters.
            #[allow(clippy::all)]
            pub mod builder {
                use super::types;
                #[allow(unused_imports)]
                use super::{
                    encode_path,
                    ByteStream,
                    ClientInfo,
                    ClientHooks,
                    Error,
                    OperationInfo,
                    RequestBuilderExt,
                    ResponseValue,
                };

                #(#builder_extra_types)*

                #(#builder_struct)*
            }

            /// Items consumers will typically use such as the Client and
            /// extension traits.
            pub mod prelude {
                #[allow(unused_imports)]
                pub use super::Client;
                #trait_preludes
            }
        };

        Ok(out)
    }

    /// Get the [`TypeSpace`] for schemas present in the `OpenAPI` specification.
    #[must_use]
    pub fn get_type_space(&self) -> &TypeSpace {
        &self.type_space
    }

    /// Whether the generated client needs to use additional crates to support
    /// futures.
    #[must_use]
    pub fn uses_futures(&self) -> bool {
        self.uses_futures
    }

    /// Whether the generated client needs to use additional crates to support
    /// websockets.
    #[must_use]
    pub fn uses_websockets(&self) -> bool {
        self.uses_websockets
    }
}

/// Add newlines after end-braces at <= two levels of indentation.
pub fn space_out_items(content: String) -> Result<String> {
    Ok(if cfg!(not(windows)) {
        let regex = regex::Regex::new(r"(\n\s*})(\n\s{0,8}[^} ])").unwrap();
        regex.replace_all(&content, "$1\n$2").to_string()
    } else {
        let regex = regex::Regex::new(r"(\n\s*})(\r\n\s{0,8}[^} ])").unwrap();
        regex.replace_all(&content, "$1\r\n$2").to_string()
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{Error, Generator, parse_openapi_value};

    #[test]
    fn sanitized_operation_ids_are_globally_unique() {
        let spec = parse_openapi_value(json!({
            "openapi": "3.0.3",
            "info": { "title": "test", "version": "1" },
            "paths": {
                "/first": {
                    "get": {
                        "operationId": "foo",
                        "responses": { "204": { "description": "ok" } }
                    }
                },
                "/second": {
                    "get": {
                        "operationId": "foo_2",
                        "responses": { "204": { "description": "ok" } }
                    }
                },
                "/third": {
                    "get": {
                        "operationId": "Foo",
                        "responses": { "204": { "description": "ok" } }
                    }
                }
            }
        }))
        .unwrap();

        let prepared = Generator::default().prepare(&spec).unwrap();
        let operation_ids = prepared
            .raw_methods
            .iter()
            .map(|method| method.operation_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(operation_ids, ["foo", "foo_2", "foo_3"]);
    }

    #[test]
    fn test_bad_value() {
        assert_eq!(
            Error::BadValue("nope".to_string(), json! { "nope"},).to_string(),
            "unexpected value type nope: \"nope\"",
        );
    }

    #[test]
    fn test_type_error() {
        assert_eq!(
            Error::UnexpectedFormat("nope".to_string()).to_string(),
            "unexpected or unhandled format in the OpenAPI document nope",
        );
    }

    #[test]
    fn test_invalid_path() {
        assert_eq!(
            Error::InvalidPath("nope".to_string()).to_string(),
            "invalid operation path nope",
        );
    }

    #[test]
    fn test_internal_error() {
        assert_eq!(
            Error::InternalError("nope".to_string()).to_string(),
            "internal error nope",
        );
    }
}
