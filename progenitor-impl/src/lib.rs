// Copyright 2025 Oxide Computer Company

//! Core implementation for the progenitor OpenAPI client generator.

#![deny(missing_docs)]

use std::collections::{BTreeMap, HashMap};

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
mod template;
mod to_schema;
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

/// OpenAPI generator.
pub struct Generator {
    type_space: TypeSpace,
    settings: GenerationSettings,
    uses_futures: bool,
    uses_websockets: bool,
    /// Maps each component schema that extends another via a top-level
    /// `allOf: [{$ref: <parent>}, ...]` to its parent's component name.
    /// Populated in `generate_tokens` after `add_ref_types`; used by
    /// `extract_responses` to collapse sibling response types that share
    /// a common ancestor (so the generated function signature gets a
    /// single error/success type instead of crashing on the
    /// multi-distinct-kind assert downstream).
    schema_supertypes: BTreeMap<String, String>,
    /// Component-schema name → `TypeId` of the corresponding typify type
    /// after `add_ref_types`. Needed alongside `schema_supertypes` so
    /// `extract_responses` can resolve a common-ancestor schema name back
    /// to a usable `TypeId` when rewriting response items.
    schema_type_ids: BTreeMap<String, TypeId>,
}

/// Settings for [Generator].
#[derive(Default, Clone)]
pub struct GenerationSettings {
    interface: InterfaceStyle,
    tag: TagStyle,
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
#[derive(Clone, Deserialize, PartialEq, Eq)]
pub enum InterfaceStyle {
    /// Use positional style.
    Positional,
    /// Use builder style.
    Builder,
}

impl Default for InterfaceStyle {
    fn default() -> Self {
        Self::Positional
    }
}

/// Style for using the OpenAPI tags when generating names in the client.
#[derive(Clone, Deserialize)]
pub enum TagStyle {
    /// Merge tags to create names in the generated client.
    Merged,
    /// Use each tag name to create separate names in the generated client.
    Separate,
}

impl Default for TagStyle {
    fn default() -> Self {
        Self::Merged
    }
}

impl GenerationSettings {
    /// Create new generator settings with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the [InterfaceStyle].
    pub fn with_interface(&mut self, interface: InterfaceStyle) -> &mut Self {
        self.interface = interface;
        self
    }

    /// Set the [TagStyle].
    pub fn with_tag(&mut self, tag: TagStyle) -> &mut Self {
        self.tag = tag;
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
    /// See [typify::TypeSpaceSettings::with_patch].
    pub fn with_patch<S: AsRef<str>>(&mut self, type_name: S, patch: &TypePatch) -> &mut Self {
        self.patch
            .insert(type_name.as_ref().to_string(), patch.clone());
        self
    }

    /// Replace a referenced type with a named type.
    /// See [typify::TypeSpaceSettings::with_replacement].
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
    /// See [typify::TypeSpaceSettings::with_conversion].
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
    /// `x-rust-type` not explicitly specified via [Self::with_crate].
    /// See [typify::TypeSpaceSettings::with_unknown_crates].
    pub fn with_unknown_crates(&mut self, policy: UnknownPolicy) -> &mut Self {
        self.unknown_crates = policy;
        self
    }

    /// Explicitly named crates whose types may be used during generation
    /// rather than generating new types based on their schemas (base on the
    /// presence of the x-rust-type extension).
    /// See [typify::TypeSpaceSettings::with_crate].
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
    /// [typify::TypeSpaceSettings::with_map_type] documentation.
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
            schema_supertypes: Default::default(),
            schema_type_ids: Default::default(),
        }
    }
}

impl Generator {
    /// Create a new generator with default values.
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
                type_settings.with_replacement(type_name, replace_name, impls.iter().cloned());
            });
        settings
            .convert
            .iter()
            .for_each(|(schema, type_name, impls)| {
                type_settings.with_conversion(schema.clone(), type_name, impls.iter().cloned());
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
            schema_supertypes: Default::default(),
            schema_type_ids: Default::default(),
        }
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

    /// Emit a [TokenStream] containing the generated client code.
    pub fn generate_tokens(&mut self, spec: &OpenApiDocument) -> Result<TokenStream> {
        let document = &spec.0;

        self.type_space.add_ref_types(
            document
                .schemas
                .iter()
                .map(|(name, schema)| (name.clone(), schema.clone())),
        )?;

        // Build the supertype map and the schema-name → TypeId map used
        // by `extract_responses` to collapse sibling response types that
        // share a common `allOf` ancestor.
        self.schema_supertypes = crate::ir::build_schema_supertype_map(&document.schemas);
        self.schema_type_ids = self.build_schema_type_id_map(&document.schemas)?;

        let raw_methods = document
            .operations
            .iter()
            .map(|operation| self.process_operation(operation, &document.schemas))
            .collect::<Result<Vec<_>>>()?;

        let operation_code = match (&self.settings.interface, &self.settings.tag) {
            (InterfaceStyle::Positional, TagStyle::Merged) => self
                .generate_tokens_positional_merged(
                    &raw_methods,
                    self.settings.inner_type.is_some(),
                ),
            (InterfaceStyle::Positional, TagStyle::Separate) => {
                unimplemented!("positional arguments with separate tags are currently unsupported")
            }
            (InterfaceStyle::Builder, TagStyle::Merged) => self
                .generate_tokens_builder_merged(&raw_methods, self.settings.inner_type.is_some()),
            (InterfaceStyle::Builder, TagStyle::Separate) => {
                let tag_info = document
                    .tags
                    .iter()
                    .map(|tag| (&tag.name, tag))
                    .collect::<BTreeMap<_, _>>();
                self.generate_tokens_builder_separate(
                    &raw_methods,
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
                s.push_str(ss);
            }
            if let Some(ss) = &document.info.terms_of_service {
                s.push_str("\n\n");
                s.push_str(ss);
            }

            s.push_str(&format!("\n\nVersion: {}", &document.info.version));

            s
        };

        let version_str = &document.info.version;

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
        };

        Ok(file)
    }

    fn generate_tokens_positional_merged(
        &mut self,
        input_methods: &[method::OperationMethod],
        has_inner: bool,
    ) -> Result<TokenStream> {
        let pairs = input_methods
            .iter()
            .map(|method| self.positional_method(method, has_inner))
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
        input_methods: &[method::OperationMethod],
        has_inner: bool,
    ) -> Result<TokenStream> {
        let pairs = input_methods
            .iter()
            .map(|method| self.builder_struct(method, TagStyle::Merged, has_inner))
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
        input_methods: &[method::OperationMethod],
        tag_info: BTreeMap<&String, &ir::Tag>,
        has_inner: bool,
    ) -> Result<TokenStream> {
        let pairs = input_methods
            .iter()
            .map(|method| self.builder_struct(method, TagStyle::Separate, has_inner))
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

    /// Get the [TypeSpace] for schemas present in the OpenAPI specification.
    pub fn get_type_space(&self) -> &TypeSpace {
        &self.type_space
    }

    /// Whether the generated client needs to use additional crates to support
    /// futures.
    pub fn uses_futures(&self) -> bool {
        self.uses_futures
    }

    /// Whether the generated client needs to use additional crates to support
    /// websockets.
    pub fn uses_websockets(&self) -> bool {
        self.uses_websockets
    }
}

/// Add newlines after end-braces at <= two levels of indentation.
pub fn space_out_items(content: String) -> Result<String> {
    Ok(if cfg!(not(windows)) {
        let regex = regex::Regex::new(r#"(\n\s*})(\n\s{0,8}[^} ])"#).unwrap();
        regex.replace_all(&content, "$1\n$2").to_string()
    } else {
        let regex = regex::Regex::new(r#"(\n\s*})(\r\n\s{0,8}[^} ])"#).unwrap();
        regex.replace_all(&content, "$1\r\n$2").to_string()
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::Error;

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
