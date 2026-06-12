// Copyright 2025 Oxide Computer Company

//! typify backend implementation.

#![deny(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};

use conversions::SchemaCache;
use log::{debug, info};
use output::OutputSpace;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use schemars::schema::{Metadata, RootSchema, Schema};
use thiserror::Error;
use type_entry::{
    StructPropertyState, TypeEntry, TypeEntryDetails, TypeEntryNative, TypeEntryNewtype,
    WrappedValue,
};

use crate::util::{sanitize, Case};

pub use crate::util::accept_as_ident;

#[cfg(test)]
mod test_util;

mod conversions;
mod convert;
mod cycles;
mod defaults;
mod enums;
mod merge;
mod output;
mod rust_extension;
mod structs;
mod type_entry;
mod util;
mod validate;
mod value;

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("unexpected value type")]
    BadValue(String, serde_json::Value),
    #[error("invalid TypeId")]
    InvalidTypeId,
    #[error("value does not conform to the given schema")]
    InvalidValue,
    #[error("invalid schema for {}: {reason}", show_type_name(.type_name.as_deref()))]
    InvalidSchema {
        type_name: Option<String>,
        reason: String,
    },
}

impl Error {
    fn invalid_value() -> Self {
        Self::InvalidValue
    }
}

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, Error>;

fn show_type_name(type_name: Option<&str>) -> &str {
    type_name.unwrap_or("<unknown type>")
}

/// Representation of a type which may have a definition or may be built-in.
#[derive(Debug)]
pub struct Type<'a> {
    type_space: &'a TypeSpace,
    type_entry: &'a TypeEntry,
}

#[allow(missing_docs)]
/// Type details returned by Type::details() to inspect a type.
pub enum TypeDetails<'a> {
    Enum(TypeEnum<'a>),
    Struct(TypeStruct<'a>),
    Newtype(TypeNewtype<'a>),

    Option(TypeId),
    Vec(TypeId),
    Map(TypeId, TypeId),
    Set(TypeId),
    Box(TypeId),
    Tuple(Box<dyn Iterator<Item = TypeId> + 'a>),
    Array(TypeId, usize),
    Builtin(&'a str),

    Unit,
    String,
}

/// Enum type details.
pub struct TypeEnum<'a> {
    details: &'a type_entry::TypeEntryEnum,
}

/// Enum variant details.
pub enum TypeEnumVariant<'a> {
    /// Variant with no associated data.
    Simple,
    /// Tuple-type variant with at least one associated type.
    Tuple(Vec<TypeId>),
    /// Struct-type variant with named properties and types.
    Struct(Vec<(&'a str, TypeId)>),
}

/// Full information pertaining to an enum variant.
pub struct TypeEnumVariantInfo<'a> {
    /// Name.
    pub name: &'a str,
    /// Description.
    pub description: Option<&'a str>,
    /// Details for the enum variant.
    pub details: TypeEnumVariant<'a>,
}

/// Struct type details.
pub struct TypeStruct<'a> {
    details: &'a type_entry::TypeEntryStruct,
}

/// Full information pertaining to a struct property.
pub struct TypeStructPropInfo<'a> {
    /// Name.
    pub name: &'a str,
    /// Description.
    pub description: Option<&'a str>,
    /// Whether the propertty is required.
    pub required: bool,
    /// Identifies the schema for the property.
    pub type_id: TypeId,
}

/// Newtype details.
pub struct TypeNewtype<'a> {
    details: &'a type_entry::TypeEntryNewtype,
}

/// Type identifier returned from type creation and used to lookup types.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Hash)]
pub struct TypeId(u64);

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Name {
    Required(String),
    Suggested(String),
    Unknown,
}

impl Name {
    pub fn into_option(self) -> Option<String> {
        match self {
            Name::Required(s) | Name::Suggested(s) => Some(s),
            Name::Unknown => None,
        }
    }

    pub fn append(&self, s: &str) -> Self {
        match self {
            Name::Required(prefix) | Name::Suggested(prefix) => {
                Self::Suggested(format!("{}_{}", prefix, s))
            }
            Name::Unknown => Name::Unknown,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum RefKey {
    Root,
    Def(String),
}

/// A collection of types.
#[derive(Debug)]
pub struct TypeSpace {
    next_id: u64,

    // TODO we need this in order to inspect the collection of reference types
    // e.g. to do `all_mutually_exclusive`. In the future, we could obviate the
    // need this by keeping a single Map of referenced types whose value was an
    // enum of a "raw" or a "converted" schema.
    definitions: BTreeMap<RefKey, Schema>,

    id_to_entry: BTreeMap<TypeId, TypeEntry>,
    type_to_id: BTreeMap<TypeEntryDetails, TypeId>,

    name_to_id: BTreeMap<String, TypeId>,
    ref_to_id: BTreeMap<RefKey, TypeId>,

    uses_chrono: bool,
    uses_uuid: bool,
    uses_serde_json: bool,
    uses_regress: bool,

    settings: TypeSpaceSettings,

    cache: SchemaCache,

    // Shared functions for generating default values
    defaults: BTreeSet<DefaultImpl>,
}

impl Default for TypeSpace {
    fn default() -> Self {
        Self {
            next_id: 1,
            definitions: Default::default(),
            id_to_entry: Default::default(),
            type_to_id: Default::default(),
            name_to_id: Default::default(),
            ref_to_id: Default::default(),
            uses_chrono: Default::default(),
            uses_uuid: Default::default(),
            uses_serde_json: Default::default(),
            uses_regress: Default::default(),
            settings: Default::default(),
            cache: Default::default(),
            defaults: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DefaultImpl {
    Boolean,
    I64,
    U64,
    NZU64,
}

/// Type name to use in generated code.
#[derive(Clone)]
pub struct MapType(pub syn::Type);

impl MapType {
    /// Create a new MapType from a [`str`].
    pub fn new(s: &str) -> Self {
        let map_type = syn::parse_str::<syn::Type>(s).expect("valid ident");
        Self(map_type)
    }
}

impl Default for MapType {
    fn default() -> Self {
        Self::new("::std::collections::HashMap")
    }
}

impl std::fmt::Debug for MapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MapType({})", self.0.to_token_stream())
    }
}

impl std::fmt::Display for MapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.to_token_stream().fmt(f)
    }
}

impl<'de> serde::Deserialize<'de> for MapType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

impl From<String> for MapType {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl From<&str> for MapType {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<syn::Type> for MapType {
    fn from(t: syn::Type) -> Self {
        Self(t)
    }
}

/// Settings that alter type generation.
#[derive(Default, Debug, Clone)]
pub struct TypeSpaceSettings {
    type_mod: Option<String>,
    extra_derives: Vec<String>,
    extra_attrs: Vec<String>,
    struct_builder: bool,

    unknown_crates: UnknownPolicy,
    crates: BTreeMap<String, CrateSpec>,
    map_type: MapType,

    patch: BTreeMap<String, TypeSpacePatch>,
    replace: BTreeMap<String, TypeSpaceReplace>,
    convert: Vec<TypeSpaceConversion>,
}

#[derive(Debug, Clone)]
struct CrateSpec {
    version: CrateVers,
    rename: Option<String>,
}

/// Policy to apply to external types described by schema extensions whose
/// crates are not explicitly specified.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize)]
pub enum UnknownPolicy {
    /// Generate the type rather according to the schema.
    #[default]
    Generate,
    /// Use the specified type by path (this will result in a compile error if
    /// one of the crates is not an existing dependency). Note that this
    /// ignores compatibility requirements specified by the schema extension
    /// and may result in subtle failures if the crate used is incompatible
    /// with the version that produced the schema.
    Allow,
    /// If an unknown crate is encountered, generate a compiler warning
    /// indicating the crate that must be specified to proceed along with
    /// version constraints. This affords users an opportunity to specify the
    /// specific crate version to use (or the user may explicitly deny use of
    /// that crate).
    Deny,
}

/// Specify the version for a named crate to consider for type use (rather than
/// generating types) in the presense of a schema extension.
#[derive(Debug, Clone)]
pub enum CrateVers {
    /// An explicit version.
    Version(semver::Version),
    /// Any version.
    Any,
    /// Never use the given crate.
    Never,
}

impl CrateVers {
    /// Parse from a string
    pub fn parse(s: &str) -> Option<Self> {
        if s == "!" {
            Some(Self::Never)
        } else if s == "*" {
            Some(Self::Any)
        } else {
            Some(Self::Version(semver::Version::parse(s).ok()?))
        }
    }
}

/// Contains a set of modifications that may be applied to an existing type.
#[derive(Debug, Default, Clone)]
pub struct TypeSpacePatch {
    rename: Option<String>,
    derives: Vec<String>,
    attrs: Vec<String>,
}

/// Contains the attributes of a replacement of an existing type.
#[derive(Debug, Default, Clone)]
pub struct TypeSpaceReplace {
    replace_type: String,
    impls: Vec<TypeSpaceImpl>,
}

/// Defines a schema which will be replaced, and the attributes of the
/// replacement.
#[derive(Debug, Clone)]
struct TypeSpaceConversion {
    schema: schemars::schema::SchemaObject,
    type_name: String,
    impls: Vec<TypeSpaceImpl>,
}

#[allow(missing_docs)]
// TODO we can currently only address traits for which cycle analysis is not
// required.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeSpaceImpl {
    FromStr,
    FromStringIrrefutable,
    Display,
    Default,
}

impl std::str::FromStr for TypeSpaceImpl {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "FromStr" => Ok(Self::FromStr),
            "Display" => Ok(Self::Display),
            "Default" => Ok(Self::Default),
            _ => Err(format!("{} is not a valid trait specifier", s)),
        }
    }
}

impl TypeSpaceSettings {
    /// Set the name of the path prefix for types defined in this [TypeSpace].
    pub fn with_type_mod<S: AsRef<str>>(&mut self, type_mod: S) -> &mut Self {
        self.type_mod = Some(type_mod.as_ref().to_string());
        self
    }

    /// Add an additional derive macro to apply to all defined types.
    pub fn with_derive(&mut self, derive: String) -> &mut Self {
        if !self.extra_derives.contains(&derive) {
            self.extra_derives.push(derive);
        }
        self
    }

    /// Add an additional attribute to apply to all defined types.
    pub fn with_attr(&mut self, attr: String) -> &mut Self {
        if !self.extra_attrs.contains(&attr) {
            self.extra_attrs.push(attr);
        }
        self
    }

    /// For structs, include a "builder" type that can be used to construct it.
    pub fn with_struct_builder(&mut self, struct_builder: bool) -> &mut Self {
        self.struct_builder = struct_builder;
        self
    }

    /// Replace a referenced type with a named type. This causes the referenced
    /// type *not* to be generated. If the same `type_name` is specified multiple times,
    /// the last one is honored.
    pub fn with_replacement<TS: ToString, RS: ToString, I: Iterator<Item = TypeSpaceImpl>>(
        &mut self,
        type_name: TS,
        replace_type: RS,
        impls: I,
    ) -> &mut Self {
        self.replace.insert(
            type_name.to_string(),
            TypeSpaceReplace {
                replace_type: replace_type.to_string(),
                impls: impls.collect(),
            },
        );
        self
    }

    /// Modify a type with the given name. Note that specifying a type not
    /// created by the input JSON schema does **not** result in an error and is
    /// silently ignored. If the same `type_name` is specified multiple times,
    /// the last one is honored.
    pub fn with_patch<S: ToString>(
        &mut self,
        type_name: S,
        type_patch: &TypeSpacePatch,
    ) -> &mut Self {
        self.patch.insert(type_name.to_string(), type_patch.clone());
        self
    }

    /// Replace a given schema with a named type. The given schema must precisely
    /// match the schema from the input, including fields such as `description`.
    /// Typical usage is to map a schema definition to a builtin type or type
    /// provided by a crate, such as `'rust_decimal::Decimal'`. If the same schema
    /// is specified multiple times, the first one is honored.
    ///
    /// # Examples
    ///
    /// ```
    /// // Setup 'number' json type to be translated into 'rust_decimal::Decimal'
    /// use schemars::schema::{InstanceType, SchemaObject};
    /// use typify_impl::{TypeSpace, TypeSpaceImpl, TypeSpaceSettings};
    /// let mut type_space = TypeSpace::new(
    ///        TypeSpaceSettings::default()
    ///            .with_struct_builder(true)
    ///            .with_conversion(
    ///                SchemaObject {
    ///                    instance_type: Some(InstanceType::Number.into()),
    ///                    ..Default::default()
    ///                },
    ///                "::rust_decimal::Decimal",
    ///                [TypeSpaceImpl::Display].into_iter(),
    ///            ),
    ///    );
    /// ```
    pub fn with_conversion<S: ToString, I: Iterator<Item = TypeSpaceImpl>>(
        &mut self,
        schema: schemars::schema::SchemaObject,
        type_name: S,
        impls: I,
    ) -> &mut Self {
        self.convert.push(TypeSpaceConversion {
            schema,
            type_name: type_name.to_string(),
            impls: impls.collect(),
        });
        self
    }

    /// Type schemas may contain an extension (`x-rust-type`) that indicates
    /// the corresponding Rust type within a particular crate. This function
    /// changes the disposition regarding crates not otherwise specified via
    /// [`Self::with_crate`]. The default value is `false`.
    pub fn with_unknown_crates(&mut self, policy: UnknownPolicy) -> &mut Self {
        self.unknown_crates = policy;
        self
    }

    /// Type schemas may contain an extension (`x-rust-type`) that indicates
    /// the corresponding Rust type within a particular crate. This extension
    /// indicates the crate, version compatibility, type path, and type
    /// parameters. This function modifies settings to use (rather than
    /// generate) types from the given crate and version. The version should
    /// precisely match the version of the crate that you expect as a
    /// dependency.
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

    /// Specify the map-like type to be used in generated code.
    ///
    /// ## Requirements
    ///
    /// - An `is_empty` method that returns a boolean
    /// - Two generic parameters, `K` and `V`
    /// - [`Default`] + [`Clone`] + [`Debug`] +
    ///   [`Serialize`][serde::Serialize] + [`Deserialize`][serde::Deserialize]
    ///
    /// ## Examples
    ///
    /// - [`::std::collections::HashMap`]
    /// - [`::std::collections::BTreeMap`]
    /// - [`::indexmap::IndexMap`](https://docs.rs/indexmap/latest/indexmap/map/struct.IndexMap.html)
    pub fn with_map_type<T: Into<MapType>>(&mut self, map_type: T) -> &mut Self {
        self.map_type = map_type.into();
        self
    }
}

impl TypeSpacePatch {
    /// Specify the new name for patched type.
    pub fn with_rename<S: ToString>(&mut self, rename: S) -> &mut Self {
        self.rename = Some(rename.to_string());
        self
    }

    /// Specify an additional derive to apply to the patched type.
    pub fn with_derive<S: ToString>(&mut self, derive: S) -> &mut Self {
        self.derives.push(derive.to_string());
        self
    }

    /// Specify an additional attribute to apply to the patched type.
    pub fn with_attr<S: ToString>(&mut self, attr: S) -> &mut Self {
        self.attrs.push(attr.to_string());
        self
    }
}

impl TypeSpace {
    /// Create a new TypeSpace with custom settings.
    pub fn new(settings: &TypeSpaceSettings) -> Self {
        let mut cache = SchemaCache::default();

        settings.convert.iter().for_each(
            |TypeSpaceConversion {
                 schema,
                 type_name,
                 impls,
             }| {
                cache.insert(schema, type_name, impls);
            },
        );

        Self {
            settings: settings.clone(),
            cache,
            ..Default::default()
        }
    }

    /// Add a collection of types that will be used as references. Regardless
    /// of how these types are defined--*de novo* or built-in--each type will
    /// appear in the final output as a struct, enum or newtype. This method
    /// may be called multiple times, but collections of references must be
    /// self-contained; in other words, a type in one invocation may not refer
    /// to a type in another invocation.
    // TODO on an error the TypeSpace is in a weird state; we, perhaps, create
    // a child TypeSpace and then merge it in once all conversions hae
    // succeeded.
    pub fn add_ref_types<I, S>(&mut self, type_defs: I) -> Result<()>
    where
        I: IntoIterator<Item = (S, Schema)>,
        S: AsRef<str>,
    {
        self.add_ref_types_impl(
            type_defs
                .into_iter()
                .map(|(key, schema)| (RefKey::Def(key.as_ref().to_string()), schema)),
        )
    }

    fn add_ref_types_impl<I>(&mut self, type_defs: I) -> Result<()>
    where
        I: IntoIterator<Item = (RefKey, Schema)>,
    {
        // Gather up all types to make things a little more convenient.
        let mut definitions = type_defs.into_iter().collect::<Vec<_>>();
        discriminator_to_oneof_prepass(&mut definitions);

        // Assign IDs to reference types before actually converting them. We'll
        // need these in the case of forward (or circular) references.
        let base_id = self.next_id;
        let def_len = definitions.len() as u64;
        self.next_id += def_len;

        for (index, (ref_name, schema)) in definitions.iter().enumerate() {
            self.ref_to_id
                .insert(ref_name.clone(), TypeId(base_id + index as u64));
            self.definitions.insert(ref_name.clone(), schema.clone());
        }

        // Convert all types; note that we use the type id assigned from the
        // previous step because each type may create additional types. This
        // effectively is doing the work of `add_type_with_name` but for a
        // batch of types.
        for (index, (ref_name, schema)) in definitions.into_iter().enumerate() {
            info!(
                "converting type: {:?} with schema {}",
                ref_name,
                serde_json::to_string(&schema).unwrap()
            );

            // Check for manually replaced types. Proceed with type conversion
            // if there is none; use the specified type if there is.
            let type_id = TypeId(base_id + index as u64);

            let maybe_replace = match &ref_name {
                RefKey::Root => None,
                RefKey::Def(def_name) => {
                    let check_name = sanitize(def_name, Case::Pascal);
                    self.settings.replace.get(&check_name)
                }
            };

            match maybe_replace {
                None => {
                    let type_name = if let RefKey::Def(name) = ref_name {
                        Name::Required(name.clone())
                    } else {
                        Name::Unknown
                    };
                    self.convert_ref_type(type_name, schema, type_id)?
                }

                Some(replace_type) => {
                    let type_entry = TypeEntry::new_native(
                        replace_type.replace_type.clone(),
                        &replace_type.impls.clone(),
                    );
                    self.id_to_entry.insert(type_id, type_entry);
                }
            }
        }

        // Eliminate cycles. It's sufficient to only start from referenced
        // types as a reference is required to make a cycle.
        self.break_cycles(base_id..base_id + def_len);

        // Finalize all created types.
        for index in base_id..self.next_id {
            let type_id = TypeId(index);
            let mut type_entry = self.id_to_entry.get(&type_id).unwrap().clone();
            debug!("finalizing type entry: {} {:#?}", index, &type_entry);
            type_entry.finalize(self)?;
            self.id_to_entry.insert(type_id, type_entry);
        }

        Ok(())
    }

    fn convert_ref_type(&mut self, type_name: Name, schema: Schema, type_id: TypeId) -> Result<()> {
        let (mut type_entry, metadata) = self.convert_schema(type_name.clone(), &schema)?;
        let default = metadata
            .as_ref()
            .and_then(|m| m.default.as_ref())
            .cloned()
            .map(WrappedValue::new);
        let type_entry = match &mut type_entry.details {
            // The types that are already named are good to go.
            TypeEntryDetails::Enum(details) => {
                details.default = default;
                type_entry
            }
            TypeEntryDetails::Struct(details) => {
                details.default = default;
                type_entry
            }
            TypeEntryDetails::Newtype(details) => {
                details.default = default;
                type_entry
            }

            // If the type entry is a reference, then this definition is a
            // simple alias to another type in this list of definitions
            // (which may nor may not have already been converted). We
            // simply create a newtype with that type ID.
            TypeEntryDetails::Reference(type_id) => TypeEntryNewtype::from_metadata(
                self,
                type_name,
                metadata,
                type_id.clone(),
                schema.clone(),
            ),

            TypeEntryDetails::Native(native) if native.name_match(&type_name) => type_entry,

            // For types that don't have names, this is effectively a type
            // alias which we treat as a newtype.
            _ => {
                info!(
                    "type alias {:?} {}\n{:?}",
                    type_name,
                    serde_json::to_string_pretty(&schema).unwrap(),
                    metadata
                );
                let subtype_id = self.assign_type(type_entry);
                TypeEntryNewtype::from_metadata(
                    self,
                    type_name,
                    metadata,
                    subtype_id,
                    schema.clone(),
                )
            }
        };
        // The component-schema name we're about to register may already
        // have been claimed by an inline schema whose heuristic-derived
        // name (e.g. `<parent>_<prop>` from a struct property) Pascal-cased
        // to the same identifier. In that case the inline entry has to
        // yield — the OpenAPI author explicitly named this component, the
        // inline-oneOf only borrowed the name from a property-based
        // inference. Rather than overwrite `name_to_id` and leave two
        // entries with the same name in `id_to_entry` (which produces a
        // duplicate-definition error at emit time), rename the inline
        // entry first.
        if let Some(entry_name) = type_entry.name() {
            if let Some(prior_id) = self.name_to_id.get(entry_name).cloned() {
                if prior_id != type_id {
                    let new_name = self.next_disambiguated_name(entry_name);
                    if let Some(prior_entry) = self.id_to_entry.get_mut(&prior_id) {
                        prior_entry.rename(new_name.clone());
                    }
                    self.name_to_id.remove(entry_name);
                    self.name_to_id.insert(new_name, prior_id);
                }
            }
            self.name_to_id.insert(entry_name.clone(), type_id.clone());
        }
        self.id_to_entry.insert(type_id, type_entry);
        Ok(())
    }

    /// Pick a name based on `base` that is not currently registered in
    /// `name_to_id`, starting with `<base>Variant` and falling back to
    /// numeric suffixes if even that is taken (extremely unlikely in
    /// practice). Used by `convert_ref_type` to rename an inline schema
    /// whose property-derived name collides with a real component name.
    fn next_disambiguated_name(&self, base: &str) -> String {
        let primary = format!("{base}Variant");
        if !self.name_to_id.contains_key(&primary) {
            return primary;
        }
        for n in 2u32.. {
            let candidate = format!("{base}Variant{n}");
            if !self.name_to_id.contains_key(&candidate) {
                return candidate;
            }
        }
        unreachable!("u32 range exhausted")
    }

    /// Add a new type and return a type identifier that may be used in
    /// function signatures or embedded within other types.
    pub fn add_type(&mut self, schema: &Schema) -> Result<TypeId> {
        self.add_type_with_name(schema, None)
    }

    /// Add a new type with a name hint and return a the components necessary
    /// to use the type for various components of a function signature.
    pub fn add_type_with_name(
        &mut self,
        schema: &Schema,
        name_hint: Option<String>,
    ) -> Result<TypeId> {
        let base_id = self.next_id;

        let name = match name_hint {
            Some(s) => Name::Suggested(s),
            None => Name::Unknown,
        };
        let (type_id, _) = self.id_for_schema(name, schema)?;

        // Finalize all created types.
        for index in base_id..self.next_id {
            let type_id = TypeId(index);
            let mut type_entry = self.id_to_entry.get(&type_id).unwrap().clone();
            type_entry.finalize(self)?;
            self.id_to_entry.insert(type_id, type_entry);
        }

        Ok(type_id)
    }

    /// Add all the types contained within a RootSchema including any
    /// referenced types and the top-level type (if there is one and it has a
    /// title).
    pub fn add_root_schema(&mut self, schema: RootSchema) -> Result<Option<TypeId>> {
        let RootSchema {
            meta_schema: _,
            schema,
            definitions,
        } = schema;

        let mut defs = definitions
            .into_iter()
            .map(|(key, schema)| (RefKey::Def(key), schema))
            .collect::<Vec<_>>();

        // Does the root type have a name (otherwise... ignore it)
        let root_type = schema
            .metadata
            .as_ref()
            .and_then(|m| m.title.as_ref())
            .is_some();

        if root_type {
            defs.push((RefKey::Root, schema.into()));
        }

        self.add_ref_types_impl(defs)?;

        if root_type {
            Ok(self.ref_to_id.get(&RefKey::Root).cloned())
        } else {
            Ok(None)
        }
    }

    /// Get a type given its ID.
    pub fn get_type(&self, type_id: &TypeId) -> Result<Type<'_>> {
        let type_entry = self.id_to_entry.get(type_id).ok_or(Error::InvalidTypeId)?;
        Ok(Type {
            type_space: self,
            type_entry,
        })
    }

    /// Whether the generated code needs `chrono` crate.
    pub fn uses_chrono(&self) -> bool {
        self.uses_chrono
    }

    /// Whether the generated code needs [regress] crate.
    pub fn uses_regress(&self) -> bool {
        self.uses_regress
    }

    /// Whether the generated code needs [serde_json] crate.
    pub fn uses_serde_json(&self) -> bool {
        self.uses_serde_json
    }

    /// Whether the generated code needs `uuid` crate.
    pub fn uses_uuid(&self) -> bool {
        self.uses_uuid
    }

    /// Iterate over all types including those defined in this [TypeSpace] and
    /// those referred to by those types.
    pub fn iter_types(&self) -> impl Iterator<Item = Type<'_>> {
        self.id_to_entry.values().map(move |type_entry| Type {
            type_space: self,
            type_entry,
        })
    }

    /// All code for processed types.
    pub fn to_stream(&self) -> TokenStream {
        let mut output = OutputSpace::default();

        // Add the error type we use for conversions; it's fine if this is
        // unused.
        output.add_item(
            output::OutputSpaceMod::Error,
            "",
            quote! {
                /// Error from a `TryFrom` or `FromStr` implementation.
                pub struct ConversionError(::std::borrow::Cow<'static, str>);

                impl ::std::error::Error for ConversionError {}
                impl ::std::fmt::Display for ConversionError {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>)
                        -> Result<(), ::std::fmt::Error>
                    {
                        ::std::fmt::Display::fmt(&self.0, f)
                    }
                }

                impl ::std::fmt::Debug for ConversionError {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>)
                        -> Result<(), ::std::fmt::Error>
                    {
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
            },
        );

        // Add all types.
        self.id_to_entry
            .values()
            .for_each(|type_entry| type_entry.output(self, &mut output));

        // Add all shared default functions.
        self.defaults
            .iter()
            .for_each(|x| output.add_item(output::OutputSpaceMod::Defaults, "", x.into()));

        output.into_stream()
    }

    /// Allocated the next TypeId.
    fn assign(&mut self) -> TypeId {
        let id = TypeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Assign a TypeId for a TypeEntry. This handles resolving references,
    /// checking for duplicate type definitions (e.g. to make sure there aren't
    /// two conflicting types of the same name), and deduplicates various
    /// flavors of built-in types.
    fn assign_type(&mut self, ty: TypeEntry) -> TypeId {
        if let TypeEntryDetails::Reference(type_id) = ty.details {
            type_id
        } else if let Some(name) = ty.name() {
            if let Some(type_id) = self.name_to_id.get(name) {
                // TODO we'd like to verify that the type is structurally the
                // same, but the types may not be functionally equal. This is a
                // consequence of types being "finalized" after each type
                // addition. This further emphasized the need for a more
                // deliberate, multi-pass approach.
                type_id.clone()
            } else {
                let type_id = self.assign();
                self.name_to_id.insert(name.clone(), type_id.clone());
                self.id_to_entry.insert(type_id.clone(), ty);
                type_id
            }
        } else if let Some(type_id) = self.type_to_id.get(&ty.details) {
            type_id.clone()
        } else {
            let type_id = self.assign();
            self.type_to_id.insert(ty.details.clone(), type_id.clone());
            self.id_to_entry.insert(type_id.clone(), ty);
            type_id
        }
    }

    /// Convert a schema to a TypeEntry and assign it a TypeId.
    ///
    /// This is used for sub-types such as the type of an array or the types of
    /// properties of a struct.
    fn id_for_schema<'a>(
        &mut self,
        type_name: Name,
        schema: &'a Schema,
    ) -> Result<(TypeId, &'a Option<Box<Metadata>>)> {
        let (mut type_entry, metadata) = self.convert_schema(type_name, schema)?;
        if let Some(metadata) = metadata {
            let default = metadata.default.clone().map(WrappedValue::new);
            match &mut type_entry.details {
                TypeEntryDetails::Enum(details) => {
                    details.default = default;
                }
                TypeEntryDetails::Struct(details) => {
                    details.default = default;
                }
                TypeEntryDetails::Newtype(details) => {
                    details.default = default;
                }
                _ => (),
            }
        }
        let type_id = self.assign_type(type_entry);
        Ok((type_id, metadata))
    }

    /// Create an Option<T> from a pre-assigned TypeId and assign it an ID.
    fn id_to_option(&mut self, id: &TypeId) -> TypeId {
        self.assign_type(TypeEntryDetails::Option(id.clone()).into())
    }

    // Create an Option<T> from a TypeEntry by assigning it type.
    fn type_to_option(&mut self, ty: TypeEntry) -> TypeEntry {
        TypeEntryDetails::Option(self.assign_type(ty)).into()
    }

    /// Create a Box<T> from a pre-assigned TypeId and assign it an ID.
    fn id_to_box(&mut self, id: &TypeId) -> TypeId {
        self.assign_type(TypeEntryDetails::Box(id.clone()).into())
    }

    /// Look up a `$ref`-style schema reference (e.g.
    /// `#/components/schemas/Foo` or `#/definitions/Foo`) against the
    /// definitions captured by `add_ref_types`. Used by the internally-
    /// tagged enum detector so a `oneOf: [{$ref: …}, ...]` produced by
    /// the discriminator pre-pass can still match on per-subtype const
    /// properties.
    pub(crate) fn resolve_ref_schema(&self, reference: &str) -> Option<&Schema> {
        for prefix in &["#/components/schemas/", "#/definitions/", "#/"] {
            if let Some(suffix) = reference.strip_prefix(prefix) {
                if let Some(schema) = self.definitions.get(&RefKey::Def(suffix.to_string())) {
                    return Some(schema);
                }
            }
        }
        None
    }
}

/// When a definition declares an OpenAPI `discriminator` AND sibling
/// definitions extend it via a top-level `allOf: [{$ref: <base>}, ...]`,
/// rewrite the base definition as a `oneOf: [<subtype>, ...]` over those
/// subtypes and detach each subtype from the (now-replaced) base. This
/// lets the existing one-of conversion produce a proper sum-type
/// `enum Base { Subtype1(Subtype1), ... }` instead of generating only a
/// bare struct with `@type: String`, which can't carry subtype payload
/// fields.
///
/// The discriminator keyword is OpenAPI-specific (not standard JSON
/// Schema), so we look for it in `extensions`. The pre-pass is a no-op
/// for schemas that don't use the OpenAPI `discriminator` keyword.
fn discriminator_to_oneof_prepass(definitions: &mut [(RefKey, Schema)]) {
    use schemars::schema::{InstanceType, SchemaObject, SubschemaValidation};
    use std::collections::{BTreeMap, BTreeSet};

    // Collect the names of every definition so we can match `$ref`
    // strings against them regardless of whether they use the
    // `#/components/schemas/` (OpenAPI) or `#/definitions/` (JSON Schema)
    // prefix.
    let def_names: BTreeSet<String> = definitions
        .iter()
        .filter_map(|(key, _)| match key {
            RefKey::Def(name) => Some(name.clone()),
            RefKey::Root => None,
        })
        .collect();

    let strip_ref = |reference: &str| -> Option<String> {
        // Try each known prefix; the suffix must match one of our def
        // names for the ref to count.
        for prefix in &["#/components/schemas/", "#/definitions/", "#/"] {
            if let Some(suffix) = reference.strip_prefix(prefix) {
                if def_names.contains(suffix) {
                    return Some(suffix.to_string());
                }
            }
        }
        None
    };

    // Pass 1: build base -> [subtype names] map. A subtype is a definition
    // whose top-level `allOf` contains exactly one `$ref` to another
    // definition (the standard "extends" pattern). Schemas with no `allOf`,
    // multiple parent refs, or mixed inline content don't qualify.
    let mut subtypes_of: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (key, schema) in definitions.iter() {
        let RefKey::Def(subtype_name) = key else {
            continue;
        };
        let Schema::Object(obj) = schema else {
            continue;
        };
        let Some(subschemas) = obj.subschemas.as_ref() else {
            continue;
        };
        let Some(all_of) = subschemas.all_of.as_ref() else {
            continue;
        };
        let parent_refs: Vec<String> = all_of
            .iter()
            .filter_map(|member| match member {
                Schema::Object(o) => o.reference.as_deref(),
                Schema::Bool(_) => None,
            })
            .filter_map(strip_ref)
            .collect();
        if parent_refs.len() == 1 {
            subtypes_of
                .entry(parent_refs.into_iter().next().expect("len == 1 above"))
                .or_default()
                .push(subtype_name.clone());
        }
    }

    // Pass 2: collect bases that (a) declare an OpenAPI `discriminator`
    // (surfaced as the `x-discriminator` extension by the progenitor
    // adapter, since JSON Schema doesn't have a typed slot for it) and
    // (b) have subtypes. For each such base, also extract the
    // discriminator's property name + mapping (subtype-name →
    // discriminator-value) so we can stamp the const value onto each
    // subtype's @type property below. Without the const value, typify
    // would generate an untagged enum and serde would try each variant
    // by structural shape, picking the wrong one when subtype payloads
    // overlap.
    #[derive(Debug)]
    struct BaseInfo {
        property_name: String,
        // subtype_name → discriminator value, ONLY for genuine
        // allOf-subtypes. Mapping targets that don't extend the base are
        // standalone types shared with other contexts (or other bases'
        // mappings) — stamping a required const onto them would corrupt
        // every other use, so they join the union untouched.
        value_by_subtype: BTreeMap<String, String>,
        // The full variant list for the synthesized oneOf: allOf-subtypes
        // plus mapping targets that exist as definitions. MongoDB Atlas's
        // DiskBackupSnapshotExportBucketResponse maps its AWS variant
        // without an allOf backref — building the union from backrefs
        // alone silently dropped it, making every AWS payload
        // undeserializable.
        union_variants: Vec<String>,
        // Plain object bases get replaced by a synthesized oneOf over
        // `union_variants` (the allOf-inheritance style). A base that
        // already models the union itself keeps its body; only its
        // member subtypes are rewritten.
        rewrite_base: bool,
    }
    let bases_to_rewrite: BTreeMap<String, BaseInfo> = definitions
        .iter()
        .filter_map(|(key, schema)| {
            let RefKey::Def(base_name) = key else {
                return None;
            };
            let Schema::Object(obj) = schema else {
                return None;
            };
            let discriminator = obj
                .extensions
                .get("x-discriminator")
                .or_else(|| obj.extensions.get("discriminator"))?;
            // OpenAPI has two discriminator styles, and wild specs mix
            // them, so the base's own shape decides what the prepass may
            // touch.
            //
            // A plain object base (the allOf-inheritance style) is
            // replaced by a oneOf over its subtypes, and every subtype is
            // rewritten.
            //
            // A base that already models the union itself (`oneOf`/`anyOf`
            // + discriminator as dispatch metadata) keeps its body:
            // rewriting would replace the real union with a oneOf over
            // whatever definitions happen to allOf-ref the base —
            // Cloudflare's rulesets_ResponseRule "refines" its 20-variant
            // rulesets_RequestRule union exactly that way (`allOf:
            // [$ref union, {required: [...]}]`), and the rewrite destroyed
            // the union and stamped a bogus discriminator value onto the
            // refinement. But subtypes that ARE members of the union still
            // need the usual treatment (MongoDB Atlas lists each variant
            // in the base's oneOf AND has the variant allOf-ref the base
            // back): stripping the backref breaks the reference cycle that
            // otherwise recurses typify's allOf merge into a stack
            // overflow, and stamping the mapped const turns the union into
            // a properly tagged enum. Non-member refiners are left alone.
            let union_members: Option<BTreeSet<String>> = obj
                .subschemas
                .as_ref()
                .and_then(|sub| sub.one_of.as_ref().or(sub.any_of.as_ref()))
                .map(|members| {
                    members
                        .iter()
                        .filter_map(|member| match member {
                            Schema::Object(o) => o.reference.as_deref(),
                            Schema::Bool(_) => None,
                        })
                        .filter_map(strip_ref)
                        .collect()
                });
            let subtypes: Vec<String> = match &union_members {
                Some(members) => subtypes_of
                    .get(base_name)?
                    .iter()
                    .filter(|subtype| members.contains(*subtype))
                    .cloned()
                    .collect(),
                None => subtypes_of.get(base_name)?.clone(),
            };
            if subtypes.is_empty() {
                return None;
            }
            let property_name = discriminator
                .get("propertyName")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("@type")
                .to_string();
            // Invert the mapping. OpenAPI's `discriminator.mapping` is
            // `{ discriminator_value: ref_path }` — flip to `subtype_name
            // → discriminator_value` for lookup below. If no explicit
            // mapping, fall back to using the subtype name itself as the
            // discriminator value.
            let mut mapped: Vec<(String, String)> = Vec::new();
            if let Some(mapping) = discriminator
                .get("mapping")
                .and_then(serde_json::Value::as_object)
            {
                for (disc_value, ref_path) in mapping {
                    if let Some(ref_str) = ref_path.as_str() {
                        if let Some(subtype_name) = ref_str
                            .strip_prefix("#/components/schemas/")
                            .or_else(|| ref_str.strip_prefix("#/definitions/"))
                            .or_else(|| ref_str.strip_prefix("#/"))
                        {
                            mapped.push((subtype_name.to_string(), disc_value.clone()));
                        }
                    }
                }
            }
            // Stamps go to genuine allOf-subtypes only; a definition the
            // mapping merely points at is shared with other contexts and
            // must not be mutated.
            let mut value_by_subtype: BTreeMap<String, String> = BTreeMap::new();
            for subtype in &subtypes {
                let value = mapped
                    .iter()
                    .find(|(name, _)| name == subtype)
                    .map(|(_, value)| value.clone())
                    .unwrap_or_else(|| subtype.clone());
                value_by_subtype.insert(subtype.clone(), value);
            }
            // The synthesized union covers the allOf-subtypes plus every
            // mapping target that exists as a definition (deterministic:
            // backref order, then mapping order).
            let mut union_variants = subtypes.clone();
            for (name, _) in &mapped {
                if def_names.contains(name) && !union_variants.contains(name) {
                    union_variants.push(name.clone());
                }
            }
            Some((
                base_name.clone(),
                BaseInfo {
                    property_name,
                    value_by_subtype,
                    union_variants,
                    rewrite_base: union_members.is_none(),
                },
            ))
        })
        .collect();

    // A discriminated base that is itself a variant or subtype of another
    // discriminated base would have its body replaced by pass 3a while the
    // parent still needs it — chained unions corrupt the whole cluster
    // (tag dispatch degrades to untagged, intermediate fields vanish).
    // Leave such bases, and thereby their sub-clusters, untouched.
    let involved: BTreeSet<String> = bases_to_rewrite
        .values()
        .flat_map(|info| {
            info.value_by_subtype
                .keys()
                .chain(info.union_variants.iter())
                .cloned()
        })
        .collect();
    let bases_to_rewrite: BTreeMap<String, BaseInfo> = bases_to_rewrite
        .into_iter()
        .filter(|(name, info)| !(info.rewrite_base && involved.contains(name)))
        .collect();

    if bases_to_rewrite.is_empty() {
        return;
    }

    // A subtype's `$ref` to its base means "inherit the base's own object
    // content" — dropping the ref without substituting that content
    // silently loses the shared fields from the standalone subtype types
    // (MongoDB Atlas declares id/provisioned/… once on the base). Capture
    // the inheritable surface before pass 3a replaces plain bases.
    let shared_content_by_base: BTreeMap<String, schemars::schema::ObjectValidation> = definitions
        .iter()
        .filter_map(|(key, schema)| {
            let RefKey::Def(base_name) = key else {
                return None;
            };
            if !bases_to_rewrite.contains_key(base_name) {
                return None;
            }
            let Schema::Object(obj) = schema else {
                return None;
            };
            let object = obj.object.as_deref().cloned()?;
            Some((base_name.clone(), object))
        })
        .collect();

    // Pass 3a: rewrite each plain-object base to a `oneOf` over its
    // union variants (union-shaped bases keep their own oneOf/anyOf
    // body). `convert_one_of`'s `maybe_internally_tagged_enum` resolves
    // `$ref` subschemas against the type space's known definitions,
    // so once each subtype carries a const-valued discriminator
    // property (stamped in pass 3b below) the resulting enum gets the
    // right `#[serde(tag = "<property>")]` attribute and the variants
    // dispatch correctly at deserialise time.
    for (key, schema) in definitions.iter_mut() {
        let RefKey::Def(base_name) = key else {
            continue;
        };
        let Some(info) = bases_to_rewrite
            .get(base_name)
            .filter(|info| info.rewrite_base)
        else {
            continue;
        };
        let Schema::Object(obj) = schema else {
            continue;
        };
        let one_of: Vec<Schema> = info
            .union_variants
            .iter()
            .map(|name| {
                Schema::Object(SchemaObject::new_ref(format!(
                    "#/components/schemas/{name}"
                )))
            })
            .collect();
        let metadata = obj.metadata.take();
        *obj = SchemaObject {
            metadata,
            subschemas: Some(Box::new(SubschemaValidation {
                one_of: Some(one_of),
                ..Default::default()
            })),
            ..Default::default()
        };
    }

    // Compute subtype → (property_name, discriminator_value) for pass
    // 3b. Pass 3b only touches the subtypes of bases we rewrote — other
    // definitions (e.g. unrelated `allOf` extensions of bases without
    // `x-discriminator`) must be left alone, otherwise we'd inject a
    // spurious discriminator property into them.
    let subtypes_to_stamp: BTreeMap<String, (String, String)> = bases_to_rewrite
        .values()
        .flat_map(|info| {
            info.value_by_subtype
                .iter()
                .map(|(subtype, value)| {
                    (subtype.clone(), (info.property_name.clone(), value.clone()))
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // Pass 3b: for each subtype of a rewritten base, drop the (now stale)
    // `$ref` to the base from its `allOf` and stamp the discriminator
    // property as a `const`-valued string so typify's `oneOf` converter
    // picks the internally-tagged path (`#[serde(tag = "@type")]`)
    // instead of falling back to an untagged enum (which would dispatch
    // by structural shape and pick the wrong variant when subtype
    // payloads overlap).
    for (key, schema) in definitions.iter_mut() {
        let RefKey::Def(subtype_name) = key else {
            continue;
        };
        let Some((discriminator_property, discriminator_value)) =
            subtypes_to_stamp.get(subtype_name)
        else {
            continue;
        };
        let Schema::Object(obj) = schema else {
            continue;
        };
        let mut stripped_bases: Vec<String> = Vec::new();
        if let Some(subschemas) = obj.subschemas.as_mut() {
            if let Some(all_of) = subschemas.all_of.as_mut() {
                all_of.retain(|member| {
                    let Schema::Object(o) = member else {
                        return true;
                    };
                    let Some(reference) = o.reference.as_deref() else {
                        return true;
                    };
                    match strip_ref(reference).filter(|name| bases_to_rewrite.contains_key(name)) {
                        Some(name) => {
                            stripped_bases.push(name);
                            false
                        }
                        None => true,
                    }
                });
                // If exactly one inline `allOf` member survives (the
                // common pattern: `allOf: [{$ref: <base>}, {type: object,
                // properties: ...}]` ⇒ just the second after we strip
                // the base ref), merge its object validation into the
                // subtype itself. typify's `maybe_internally_tagged_enum`
                // requires `get_object()` to succeed, which only happens
                // when the schema has `subschemas: None`. Only a PURE
                // inline object qualifies: hoisting a surviving `$ref` (or
                // a member carrying non-object constraints) would discard
                // it outright — a mixin ref's fields must stay, even at
                // the cost of the tagged-dispatch optimisation, so
                // anything else keeps its `allOf` for typify's merge.
                let lone_pure_object = all_of.len() == 1
                    && matches!(
                        &all_of[0],
                        Schema::Object(inline) if inline.reference.is_none()
                            && inline.subschemas.is_none()
                            && inline.enum_values.is_none()
                            && inline.const_value.is_none()
                            && inline.string.is_none()
                            && inline.number.is_none()
                            && inline.array.is_none()
                    );
                if lone_pure_object {
                    if let Schema::Object(inline) = all_of[0].clone() {
                        let inline_obj = inline.object;
                        if obj.object.is_none() {
                            obj.object = Some(Box::default());
                        }
                        if let Some(target) = obj.object.as_mut() {
                            if let Some(inline_obj) = inline_obj {
                                for (name, schema) in inline_obj.properties {
                                    target.properties.entry(name).or_insert(schema);
                                }
                                for name in inline_obj.required {
                                    target.required.insert(name);
                                }
                                if target.additional_properties.is_none() {
                                    target.additional_properties = inline_obj.additional_properties;
                                }
                            }
                        }
                        if obj.instance_type.is_none() {
                            obj.instance_type = inline.instance_type;
                        }
                    }
                    subschemas.all_of = None;
                } else if all_of.is_empty() {
                    subschemas.all_of = None;
                }
            }
            if subschemas.all_of.is_none()
                && subschemas.any_of.is_none()
                && subschemas.one_of.is_none()
                && subschemas.not.is_none()
                && subschemas.if_schema.is_none()
                && subschemas.then_schema.is_none()
                && subschemas.else_schema.is_none()
            {
                obj.subschemas = None;
            }
        }
        // The stripped `$ref` meant "inherit the base's fields"; fold the
        // base's own object content into the subtype so the standalone
        // subtype type keeps the shared properties. The subtype's own
        // declarations (already merged above) take precedence, and the
        // discriminator const stamped below still overrides its property.
        for base in &stripped_bases {
            let Some(shared) = shared_content_by_base.get(base) else {
                continue;
            };
            if obj.object.is_none() {
                obj.object = Some(Box::default());
            }
            if let Some(target) = obj.object.as_mut() {
                for (name, schema) in &shared.properties {
                    target
                        .properties
                        .entry(name.clone())
                        .or_insert_with(|| schema.clone());
                }
                for name in &shared.required {
                    target.required.insert(name.clone());
                }
                if target.additional_properties.is_none() {
                    target.additional_properties = shared.additional_properties.clone();
                }
            }
        }
        // Stamp the discriminator property as a `const`-valued string.
        // typify's `maybe_internally_tagged_enum` recognises a `oneOf`
        // whose variants each declare the same required property with a
        // const value, and emits `#[serde(tag = "<property>")]`. Without
        // the const, typify falls back to an untagged enum and serde
        // dispatches by structural shape (wrong choice when payloads
        // overlap, as in our `Tilgungswunsch` case).
        //
        // Drop any prior declaration of the property (the subtype might
        // already have it as a `type: string`, e.g. from the spec
        // author's earlier copy-paste of the base's @type field) so the
        // const version takes precedence.
        if obj.object.is_none() {
            obj.object = Some(Box::default());
        }
        if let Some(object) = obj.object.as_mut() {
            let prop = Schema::Object(SchemaObject {
                instance_type: Some(InstanceType::String.into()),
                enum_values: Some(vec![serde_json::Value::String(discriminator_value.clone())]),
                ..Default::default()
            });
            object
                .properties
                .insert(discriminator_property.clone(), prop);
            object.required.insert(discriminator_property.clone());
        }
        if obj.instance_type.is_none() {
            obj.instance_type = Some(InstanceType::Object.into());
        }
    }
}

impl ToTokens for TypeSpace {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_stream())
    }
}

impl Type<'_> {
    /// The name of the type as a String.
    pub fn name(&self) -> String {
        let Type {
            type_space,
            type_entry,
        } = self;
        type_entry.type_name(type_space)
    }

    /// The identifier for the type as might be used for a function return or
    /// defining the type of a member of a struct..
    pub fn ident(&self) -> TokenStream {
        let Type {
            type_space,
            type_entry,
        } = self;
        type_entry.type_ident(type_space, &type_space.settings.type_mod)
    }

    /// The identifier for the type as might be used for a parameter in a
    /// function signature. In general: simple types are the same as
    /// [Type::ident] and complex types prepend a `&`.
    pub fn parameter_ident(&self) -> TokenStream {
        let Type {
            type_space,
            type_entry,
        } = self;
        type_entry.type_parameter_ident(type_space, None)
    }

    /// The identifier for the type as might be used for a parameter in a
    /// function signature along with a lifetime parameter. In general: simple
    /// types are the same as [Type::ident] and complex types prepend a
    /// `&'<lifetime>`.
    pub fn parameter_ident_with_lifetime(&self, lifetime: &str) -> TokenStream {
        let Type {
            type_space,
            type_entry,
        } = self;
        type_entry.type_parameter_ident(type_space, Some(lifetime))
    }

    /// A textual description of the type appropriate for debug output.
    pub fn describe(&self) -> String {
        self.type_entry.describe()
    }

    /// Get details about the type.
    pub fn details(&self) -> TypeDetails<'_> {
        match &self.type_entry.details {
            // Named user-defined types
            TypeEntryDetails::Enum(details) => TypeDetails::Enum(TypeEnum { details }),
            TypeEntryDetails::Struct(details) => TypeDetails::Struct(TypeStruct { details }),
            TypeEntryDetails::Newtype(details) => TypeDetails::Newtype(TypeNewtype { details }),

            // Compound types
            TypeEntryDetails::Option(type_id) => TypeDetails::Option(type_id.clone()),
            TypeEntryDetails::Vec(type_id) => TypeDetails::Vec(type_id.clone()),
            TypeEntryDetails::Map(key_id, value_id) => {
                TypeDetails::Map(key_id.clone(), value_id.clone())
            }
            TypeEntryDetails::Set(type_id) => TypeDetails::Set(type_id.clone()),
            TypeEntryDetails::Box(type_id) => TypeDetails::Box(type_id.clone()),
            TypeEntryDetails::Tuple(types) => TypeDetails::Tuple(Box::new(types.iter().cloned())),
            TypeEntryDetails::Array(type_id, length) => {
                TypeDetails::Array(type_id.clone(), *length)
            }

            // Builtin types
            TypeEntryDetails::Unit => TypeDetails::Unit,
            TypeEntryDetails::Native(TypeEntryNative {
                type_name: name, ..
            })
            | TypeEntryDetails::Integer(name)
            | TypeEntryDetails::Float(name) => TypeDetails::Builtin(name.as_str()),
            TypeEntryDetails::Boolean => TypeDetails::Builtin("bool"),
            TypeEntryDetails::String => TypeDetails::String,
            TypeEntryDetails::JsonValue => TypeDetails::Builtin("::serde_json::Value"),

            // Only used during processing; shouldn't be visible at this point
            TypeEntryDetails::Reference(_) => unreachable!(),
        }
    }

    /// Checks if the type has the associated impl.
    pub fn has_impl(&self, impl_name: TypeSpaceImpl) -> bool {
        let Type {
            type_space,
            type_entry,
        } = self;
        type_entry.has_impl(type_space, impl_name)
    }

    /// Provides the the type identifier for the builder if one exists.
    pub fn builder(&self) -> Option<TokenStream> {
        let Type {
            type_space,
            type_entry,
        } = self;

        if !type_space.settings.struct_builder {
            return None;
        }

        match &type_entry.details {
            TypeEntryDetails::Struct(type_entry::TypeEntryStruct { name, .. }) => {
                match &type_space.settings.type_mod {
                    Some(type_mod) => {
                        let type_mod = format_ident!("{}", type_mod);
                        let type_name = format_ident!("{}", name);
                        Some(quote! { #type_mod :: builder :: #type_name })
                    }
                    None => {
                        let type_name = format_ident!("{}", name);
                        Some(quote! { builder :: #type_name })
                    }
                }
            }
            _ => None,
        }
    }
}

impl<'a> TypeEnum<'a> {
    /// Get name and information of each enum variant.
    pub fn variants(&'a self) -> impl Iterator<Item = (&'a str, TypeEnumVariant<'a>)> {
        self.variants_info().map(|info| (info.name, info.details))
    }

    /// Get all information for each enum variant.
    pub fn variants_info(&'a self) -> impl Iterator<Item = TypeEnumVariantInfo<'a>> {
        self.details.variants.iter().map(move |variant| {
            let details = match &variant.details {
                type_entry::VariantDetails::Simple => TypeEnumVariant::Simple,
                // The distinction between a lone item variant and a tuple
                // variant with a single item is only relevant internally.
                type_entry::VariantDetails::Item(type_id) => {
                    TypeEnumVariant::Tuple(vec![type_id.clone()])
                }
                type_entry::VariantDetails::Tuple(types) => TypeEnumVariant::Tuple(types.clone()),
                type_entry::VariantDetails::Struct(properties) => TypeEnumVariant::Struct(
                    properties
                        .iter()
                        .map(|prop| (prop.name.as_str(), prop.type_id.clone()))
                        .collect(),
                ),
            };
            TypeEnumVariantInfo {
                name: variant.ident_name.as_ref().unwrap(),
                description: variant.description.as_deref(),
                details,
            }
        })
    }
}

impl<'a> TypeStruct<'a> {
    /// Get name and type of each property.
    pub fn properties(&'a self) -> impl Iterator<Item = (&'a str, TypeId)> {
        self.details
            .properties
            .iter()
            .map(move |prop| (prop.name.as_str(), prop.type_id.clone()))
    }

    /// Get all information about each struct property.
    pub fn properties_info(&'a self) -> impl Iterator<Item = TypeStructPropInfo<'a>> {
        self.details
            .properties
            .iter()
            .map(move |prop| TypeStructPropInfo {
                name: prop.name.as_str(),
                description: prop.description.as_deref(),
                required: matches!(&prop.state, StructPropertyState::Required),
                type_id: prop.type_id.clone(),
            })
    }
}

impl TypeNewtype<'_> {
    /// Get the inner type of the newtype struct.
    pub fn inner(&self) -> TypeId {
        self.details.type_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use schema::Schema;
    use schemars::{schema_for, JsonSchema};
    use serde::Serialize;
    use serde_json::json;
    use std::collections::HashSet;

    use crate::{
        output::OutputSpace,
        test_util::validate_output,
        type_entry::{TypeEntryEnum, VariantDetails},
        Name, TypeEntryDetails, TypeSpace, TypeSpaceSettings,
    };

    #[allow(dead_code)]
    #[derive(Serialize, JsonSchema)]
    struct Blah {
        blah: String,
    }

    #[allow(dead_code)]
    #[derive(Serialize, JsonSchema)]
    #[serde(rename_all = "camelCase")]
    //#[serde(untagged)]
    //#[serde(tag = "type", content = "content")]
    enum E {
        /// aaa
        A,
        /// bee
        B,
        /// cee
        //C(Vec<String>),
        C(Blah),
        /// dee
        D {
            /// double D
            dd: String,
        },
        // /// eff
        // F(
        //     /// eff.0
        //     u32,
        //     /// eff.1
        //     u32,
        // ),
    }

    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(rename_all = "camelCase")]
    struct Foo {
        /// this is bar
        #[serde(default)]
        bar: Option<String>,
        baz_baz: i32,
        /// eeeeee!
        e: E,
    }

    #[test]
    fn test_mapping_only_variants_join_the_union_unstamped() {
        // MongoDB Atlas's DiskBackupSnapshotExportBucketResponse maps its
        // AWS variant in `discriminator.mapping` but the AWS definition
        // does NOT allOf-ref the base (Azure does). The union must cover
        // both — building it from backrefs alone made every AWS payload
        // fail with "unknown variant" — and the standalone mapped-only
        // definition must NOT be mutated (it is shared with other
        // contexts; it already declares its own discriminator property).
        let definitions: serde_json::Map<String, serde_json::Value> = json!({
            "ExportBucket": {
                "type": "object",
                "x-discriminator": {
                    "propertyName": "cloudProvider",
                    "mapping": {
                        "AWS": "#/components/schemas/AwsBucket",
                        "AZURE": "#/components/schemas/AzureBucket"
                    }
                },
                "properties": {
                    "cloudProvider": { "type": "string" },
                    "bucketName": { "type": "string" }
                }
            },
            "AwsBucket": {
                "type": "object",
                "properties": {
                    "cloudProvider": { "type": "string", "enum": ["AWS"] },
                    "bucketName": { "type": "string" },
                    "iamRoleId": { "type": "string" }
                },
                "required": ["cloudProvider"]
            },
            "AzureBucket": {
                "allOf": [
                    { "$ref": "#/components/schemas/ExportBucket" },
                    {
                        "type": "object",
                        "properties": { "serviceUrl": { "type": "string" } }
                    }
                ]
            }
        })
        .as_object()
        .unwrap()
        .clone();

        let mut type_space = TypeSpace::default();
        type_space
            .add_ref_types(definitions.into_iter().map(|(name, value)| {
                (
                    name,
                    serde_json::from_value::<schemars::schema::Schema>(value).unwrap(),
                )
            }))
            .unwrap();

        let actual = type_space.to_stream().to_string();
        assert!(actual.contains("enum ExportBucket"), "{}", actual);
        // Both the backref subtype and the mapping-only variant are in.
        assert!(actual.contains("AzureBucket"), "{}", actual);
        // Tagged dispatch with both variants, including the mapped-only
        // AWS one (struct variants — slicing to the first brace would
        // stop inside Azure's body, so assert on the variant headers).
        assert!(
            actual.contains("(tag = \"cloudProvider\")] pub enum ExportBucket"),
            "{}",
            actual
        );
        let union_body = actual.split("pub enum ExportBucket").nth(1).unwrap();
        assert!(union_body.contains("\"AWS\")] Aws"), "{}", actual);
        assert!(union_body.contains("\"AZURE\")] Azure"), "{}", actual);
        // The standalone mapped-only struct keeps its own (optional)
        // bucketName — no required-const stamping beyond what it
        // declared itself.
        let aws_struct = actual
            .split("pub struct AwsBucket")
            .nth(1)
            .expect("AwsBucket struct exists");
        let aws_fields = &aws_struct[..aws_struct.find('}').unwrap()];
        assert!(aws_fields.contains("iam_role_id"), "{}", actual);
    }

    #[test]
    fn test_chained_discriminated_bases_left_untouched() {
        // A base that is itself a subtype/variant of another discriminated
        // base must not be rewritten: replacing its body with a synthesized
        // oneOf would destroy content its parent union (and its own
        // standalone uses) still need. The middle link keeps its own
        // fields; its child still inherits through the untouched chain.
        let definitions: serde_json::Map<String, serde_json::Value> = json!({
            "Node": {
                "type": "object",
                "properties": { "kind": { "type": "string" } },
                "oneOf": [{ "$ref": "#/components/schemas/Branch" }],
                "x-discriminator": { "propertyName": "kind" }
            },
            "Branch": {
                "allOf": [
                    { "$ref": "#/components/schemas/Node" },
                    {
                        "type": "object",
                        "properties": { "branchOwn": { "type": "string" } }
                    }
                ],
                "x-discriminator": { "propertyName": "branchKind" }
            },
            "Leaf": {
                "allOf": [
                    { "$ref": "#/components/schemas/Branch" },
                    {
                        "type": "object",
                        "properties": { "leafOwn": { "type": "string" } }
                    }
                ]
            }
        })
        .as_object()
        .unwrap()
        .clone();

        let mut type_space = TypeSpace::default();
        type_space
            .add_ref_types(definitions.into_iter().map(|(name, value)| {
                (
                    name,
                    serde_json::from_value::<schemars::schema::Schema>(value).unwrap(),
                )
            }))
            .unwrap();

        let actual = type_space.to_stream().to_string();
        // The middle link keeps its own field...
        let branch = actual
            .split("pub struct Branch")
            .nth(1)
            .expect("Branch struct exists");
        assert!(
            branch[..branch.find('}').unwrap()].contains("branch_own"),
            "{}",
            actual
        );
        // ...and the leaf still sees the whole inheritance chain.
        let leaf = actual
            .split("pub struct Leaf")
            .nth(1)
            .expect("Leaf struct exists");
        let leaf_fields = &leaf[..leaf.find('}').unwrap()];
        assert!(leaf_fields.contains("leaf_own"), "{}", actual);
        assert!(leaf_fields.contains("branch_own"), "{}", actual);
    }

    #[test]
    fn test_allof_inheritance_subtypes_keep_base_fields() {
        // The allOf-inheritance style: a plain object base declares the
        // discriminator and the shared fields; subtypes extend it via
        // allOf. The prepass synthesizes the union — and must fold the
        // base's fields into each subtype, since the stripped `$ref`
        // carried them.
        let definitions: serde_json::Map<String, serde_json::Value> = json!({
            "Pet": {
                "type": "object",
                "properties": {
                    "petType": { "type": "string" },
                    "name": { "type": "string" }
                },
                "required": ["petType", "name"],
                "x-discriminator": { "propertyName": "petType" }
            },
            "Dog": {
                "allOf": [
                    { "$ref": "#/components/schemas/Pet" },
                    {
                        "type": "object",
                        "properties": { "bark": { "type": "boolean" } }
                    }
                ]
            },
            "Cat": {
                "allOf": [
                    { "$ref": "#/components/schemas/Pet" },
                    {
                        "type": "object",
                        "properties": { "lives": { "type": "integer" } }
                    }
                ]
            }
        })
        .as_object()
        .unwrap()
        .clone();

        let mut type_space = TypeSpace::default();
        type_space
            .add_ref_types(definitions.into_iter().map(|(name, value)| {
                (
                    name,
                    serde_json::from_value::<schemars::schema::Schema>(value).unwrap(),
                )
            }))
            .unwrap();

        let actual = type_space.to_stream().to_string();
        assert!(actual.contains("enum Pet"), "{}", actual);
        let dog_struct = actual
            .split("pub struct Dog")
            .nth(1)
            .expect("Dog struct exists");
        let dog_fields = &dog_struct[..dog_struct.find('}').unwrap()];
        assert!(dog_fields.contains("bark"), "{}", actual);
        // Inherited from the stripped base ref.
        assert!(dog_fields.contains("name"), "{}", actual);
    }

    #[test]
    fn test_cyclic_oneof_discriminator_members_terminate() {
        // MongoDB Atlas encodes its discriminated unions BOTH ways at
        // once: the base lists every variant in `oneOf` AND each variant
        // `allOf`-refs the base back. The backref must be stripped (it
        // only re-states membership) or the reference cycle recurses the
        // allOf merge into a stack overflow; the mapped const must still
        // be stamped so the union becomes a tagged enum.
        let definitions: serde_json::Map<String, serde_json::Value> = json!({
            "CloudProviderContainer": {
                "type": "object",
                "properties": {
                    "providerName": { "type": "string" }
                },
                "oneOf": [
                    { "$ref": "#/components/schemas/AwsContainer" },
                    { "$ref": "#/components/schemas/AzureContainer" }
                ],
                "x-discriminator": {
                    "propertyName": "providerName",
                    "mapping": {
                        "AWS": "#/components/schemas/AwsContainer",
                        "AZURE": "#/components/schemas/AzureContainer"
                    }
                }
            },
            "AwsContainer": {
                "allOf": [
                    { "$ref": "#/components/schemas/CloudProviderContainer" },
                    {
                        "type": "object",
                        "properties": { "regionName": { "type": "string" } }
                    }
                ]
            },
            "AzureContainer": {
                "allOf": [
                    { "$ref": "#/components/schemas/CloudProviderContainer" },
                    {
                        "type": "object",
                        "properties": { "azureSubscriptionId": { "type": "string" } }
                    }
                ]
            }
        })
        .as_object()
        .unwrap()
        .clone();

        let mut type_space = TypeSpace::default();
        type_space
            .add_ref_types(definitions.into_iter().map(|(name, value)| {
                (
                    name,
                    serde_json::from_value::<schemars::schema::Schema>(value).unwrap(),
                )
            }))
            .unwrap();

        let actual = type_space.to_stream().to_string();
        assert!(actual.contains("enum CloudProviderContainer"), "{}", actual);
        // The mapped discriminator values drive serde dispatch.
        assert!(actual.contains("AWS"), "{}", actual);
        assert!(actual.contains("AZURE"), "{}", actual);
        // The backref strip must substitute the base's shared fields, not
        // drop them: the standalone subtype structs keep them.
        let aws_struct = actual
            .split("pub struct AwsContainer")
            .nth(1)
            .expect("AwsContainer struct exists");
        let aws_fields = &aws_struct[..aws_struct.find('}').unwrap()];
        assert!(aws_fields.contains("region_name"), "{}", actual);
        assert!(aws_fields.contains("provider_name"), "{}", actual);
    }

    #[test]
    fn test_oneof_discriminator_union_survives_refinement() {
        // Cloudflare's rulesets: the discriminated union is modelled as
        // `oneOf` + `discriminator` (dispatch-metadata style), and a
        // separate definition *refines* the whole union via
        // `allOf: [$ref union, {required: [...]}]`. The allOf-inheritance
        // prepass must not mistake that refinement for a subtype: it used
        // to replace the 20-variant union with `oneOf: [TheRefinement]`
        // and stamp the refinement with a bogus discriminator value.
        let definitions: serde_json::Map<String, serde_json::Value> = json!({
            "BlockRule": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["block"] },
                    "expression": { "type": "string" }
                },
                "required": ["action"]
            },
            "SkipRule": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["skip"] },
                    "expression": { "type": "string" }
                },
                "required": ["action"]
            },
            "RequestRule": {
                "oneOf": [
                    { "$ref": "#/components/schemas/BlockRule" },
                    { "$ref": "#/components/schemas/SkipRule" }
                ],
                "x-discriminator": {
                    "propertyName": "action",
                    "mapping": {
                        "block": "#/components/schemas/BlockRule",
                        "skip": "#/components/schemas/SkipRule"
                    }
                }
            },
            "ResponseRule": {
                "allOf": [
                    { "$ref": "#/components/schemas/RequestRule" },
                    { "required": ["expression", "action"] }
                ]
            }
        })
        .as_object()
        .unwrap()
        .clone();

        let mut type_space = TypeSpace::default();
        type_space
            .add_ref_types(definitions.into_iter().map(|(name, value)| {
                (
                    name,
                    serde_json::from_value::<schemars::schema::Schema>(value).unwrap(),
                )
            }))
            .unwrap();

        let actual = type_space.to_stream().to_string();
        // The union keeps both real variants...
        assert!(actual.contains("enum RequestRule"), "{}", actual);
        assert!(actual.contains("BlockRule"), "{}", actual);
        assert!(actual.contains("SkipRule"), "{}", actual);
        // ...and the refinement is not enrolled as a variant of it.
        assert!(
            !actual.contains("RequestRule :: ResponseRule"),
            "refinement became a union variant:\n{}",
            actual
        );
    }

    #[test]
    fn test_simple() {
        let schema = schema_for!(Foo);
        println!("{:#?}", schema);
        let mut type_space = TypeSpace::default();
        type_space.add_ref_types(schema.definitions).unwrap();
        let (ty, _) = type_space
            .convert_schema_object(
                Name::Unknown,
                &schemars::schema::Schema::Object(schema.schema.clone()),
                &schema.schema,
            )
            .unwrap();

        println!("{:#?}", ty);

        let mut output = OutputSpace::default();
        ty.output(&type_space, &mut output);
        println!("{}", output.into_stream());

        for ty in type_space.id_to_entry.values() {
            println!("{:#?}", ty);
            let mut output = OutputSpace::default();
            ty.output(&type_space, &mut output);
            println!("{}", output.into_stream());
        }
    }

    #[test]
    fn test_external_references() {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-04/schema#",
            "definitions": {
                "somename": {
                    "$ref": "#/definitions/someothername",
                    "required": [ "someproperty" ]
                },
                "someothername": {
                    "type": "object",
                    "properties": {
                        "someproperty": {
                            "type": "string"
                        }
                    }
                }
            }
        });
        let schema = serde_json::from_value(schema).unwrap();
        println!("{:#?}", schema);
        let settings = TypeSpaceSettings::default();
        let mut type_space = TypeSpace::new(&settings);
        type_space.add_root_schema(schema).unwrap();
        let tokens = type_space.to_stream().to_string();
        println!("{}", tokens);
        assert!(tokens
            .contains(" pub struct Somename { pub someproperty : :: std :: string :: String , }"))
    }

    #[test]
    fn test_convert_enum_string() {
        #[allow(dead_code)]
        #[derive(JsonSchema)]
        #[serde(rename_all = "camelCase")]
        enum SimpleEnum {
            DotCom,
            Grizz,
            Kenneth,
        }

        let schema = schema_for!(SimpleEnum);
        println!("{:#?}", schema);

        let mut type_space = TypeSpace::default();
        type_space.add_ref_types(schema.definitions).unwrap();
        let (ty, _) = type_space
            .convert_schema_object(
                Name::Unknown,
                &schemars::schema::Schema::Object(schema.schema.clone()),
                &schema.schema,
            )
            .unwrap();

        match ty.details {
            TypeEntryDetails::Enum(TypeEntryEnum { variants, .. }) => {
                for variant in &variants {
                    assert_eq!(variant.details, VariantDetails::Simple);
                }
                let var_names = variants
                    .iter()
                    .map(|variant| variant.ident_name.as_ref().unwrap().clone())
                    .collect::<HashSet<_>>();
                assert_eq!(
                    var_names,
                    ["DotCom", "Grizz", "Kenneth",]
                        .iter()
                        .map(ToString::to_string)
                        .collect::<HashSet<_>>()
                );
            }
            _ => {
                let mut output = OutputSpace::default();
                ty.output(&type_space, &mut output);
                println!("{}", output.into_stream());
                panic!();
            }
        }
    }

    #[test]
    fn test_string_enum_with_null() {
        let original_schema = json!({ "$ref": "xxx"});
        let enum_values = vec![
            json!("Shadrach"),
            json!("Meshach"),
            json!("Abednego"),
            json!(null),
        ];

        let mut type_space = TypeSpace::default();
        let (te, _) = type_space
            .convert_enum_string(
                Name::Required("OnTheGo".to_string()),
                &serde_json::from_value(original_schema).unwrap(),
                &None,
                &enum_values,
                None,
            )
            .unwrap();

        if let TypeEntryDetails::Option(id) = &te.details {
            let ote = type_space.id_to_entry.get(id).unwrap();
            if let TypeEntryDetails::Enum(TypeEntryEnum { variants, .. }) = &ote.details {
                let variants = variants
                    .iter()
                    .map(|v| match v.details {
                        VariantDetails::Simple => v.ident_name.as_ref().unwrap().clone(),
                        _ => panic!("unexpected variant type"),
                    })
                    .collect::<HashSet<_>>();

                assert_eq!(
                    variants,
                    enum_values
                        .iter()
                        .flat_map(|j| j.as_str().map(ToString::to_string))
                        .collect::<HashSet<_>>()
                );
            } else {
                panic!("not the sub-type we expected {:#?}", te)
            }
        } else {
            panic!("not the type we expected {:#?}", te)
        }
    }

    #[test]
    fn test_alias() {
        #[allow(dead_code)]
        #[derive(JsonSchema, Schema)]
        struct Stuff(Vec<String>);

        #[allow(dead_code)]
        #[derive(JsonSchema, Schema)]
        struct Things {
            a: String,
            b: Stuff,
        }

        validate_output::<Things>();
    }

    #[test]
    fn test_builder_name() {
        #[allow(dead_code)]
        #[derive(JsonSchema)]
        struct TestStruct {
            x: u32,
        }

        let mut type_space = TypeSpace::default();
        let schema = schema_for!(TestStruct);
        let type_id = type_space.add_root_schema(schema).unwrap().unwrap();
        let ty = type_space.get_type(&type_id).unwrap();

        assert!(ty.builder().is_none());

        let mut type_space = TypeSpace::new(TypeSpaceSettings::default().with_struct_builder(true));
        let schema = schema_for!(TestStruct);
        let type_id = type_space.add_root_schema(schema).unwrap().unwrap();
        let ty = type_space.get_type(&type_id).unwrap();

        assert_eq!(
            ty.builder().map(|ts| ts.to_string()),
            Some("builder :: TestStruct".to_string())
        );

        let mut type_space = TypeSpace::new(
            TypeSpaceSettings::default()
                .with_type_mod("types")
                .with_struct_builder(true),
        );
        let schema = schema_for!(TestStruct);
        let type_id = type_space.add_root_schema(schema).unwrap().unwrap();
        let ty = type_space.get_type(&type_id).unwrap();

        assert_eq!(
            ty.builder().map(|ts| ts.to_string()),
            Some("types :: builder :: TestStruct".to_string())
        );

        #[allow(dead_code)]
        #[derive(JsonSchema)]
        enum TestEnum {
            X,
            Y,
        }
        let mut type_space = TypeSpace::new(
            TypeSpaceSettings::default()
                .with_type_mod("types")
                .with_struct_builder(true),
        );
        let schema = schema_for!(TestEnum);
        let type_id = type_space.add_root_schema(schema).unwrap().unwrap();
        let ty = type_space.get_type(&type_id).unwrap();
        assert!(ty.builder().is_none());
    }

    /// Regression: an inline property whose heuristic-derived name
    /// Pascal-cases to the same identifier as a separately-named component
    /// schema must not collide. Before the fix, both would land in
    /// `id_to_entry` under the same name and the emitted Rust failed to
    /// compile with `E0428 the name is defined multiple times`. After the
    /// fix the inline entry is renamed (suffixed with `Variant`).
    #[test]
    fn test_inline_schema_renamed_when_colliding_with_named_component() {
        // The order matters: the schema that contains the inline property
        // is processed first, registering the inline-derived name in
        // `name_to_id`. When the component with the same Pascal-cased
        // name arrives, the inline entry is renamed.
        let parent = serde_json::from_value::<schemars::schema::Schema>(json!({
            "type": "object",
            "properties": {
                "vermietungErfassung": {
                    "oneOf": [
                        { "type": "object", "properties": { "kind": { "const": "keine" } } },
                        { "type": "object", "properties": { "kind": { "const": "vorhanden" } } },
                    ],
                },
            },
        }))
        .unwrap();
        let collision = serde_json::from_value::<schemars::schema::Schema>(json!({
            "type": "object",
            "required": ["@type"],
            "properties": {
                "@type": { "type": "string" },
            },
        }))
        .unwrap();

        let mut type_space = TypeSpace::new(&TypeSpaceSettings::default());
        type_space
            .add_ref_types(vec![
                ("Stellplatz".to_string(), parent),
                ("StellplatzVermietungErfassung".to_string(), collision),
            ])
            .unwrap();
        let tokens = type_space.to_stream().to_string();

        // The named component keeps the original identifier.
        assert!(
            tokens.contains("pub struct StellplatzVermietungErfassung"),
            "named component must keep its declared name; output:\n{tokens}"
        );
        // The inline-derived enum is renamed with the `Variant` suffix.
        assert!(
            tokens.contains("pub enum StellplatzVermietungErfassungVariant"),
            "inline-derived enum must be renamed on collision; output:\n{tokens}"
        );
    }
}
