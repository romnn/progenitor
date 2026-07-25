// Copyright 2026

//! Structural deduplication of named types.
//!
//! Two named types that differ only in their name share one definition; the
//! other names become `pub type` aliases. These tests pin what may and may not
//! be merged — a merge that ignored a `maxLength` or a permitted enum value
//! would be a silent validation change, not a formatting one.

use schemars::schema::RootSchema;
use typify_impl::{TypeDedup, TypeSpace, TypeSpaceSettings};

/// Top-level items declared by the generated code, as `(kind, name)`.
fn items(schema: &RootSchema, dedup: TypeDedup) -> Vec<(String, String)> {
    let mut settings = TypeSpaceSettings::default();
    settings.with_type_dedup(dedup);
    let mut type_space = TypeSpace::new(&settings);
    type_space.add_root_schema(schema.clone()).unwrap();

    let file = syn::parse2::<syn::File>(type_space.to_stream()).unwrap();
    file.items
        .iter()
        .flat_map(|item| match item {
            syn::Item::Mod(m) => m
                .content
                .iter()
                .flat_map(|(_, items)| items.iter())
                .collect::<Vec<_>>(),
            other => vec![other],
        })
        .filter_map(|item| match item {
            syn::Item::Struct(s) => Some(("struct".to_string(), s.ident.to_string())),
            syn::Item::Enum(e) => Some(("enum".to_string(), e.ident.to_string())),
            syn::Item::Use(u) => use_rename(&u.tree).map(|n| ("alias".to_string(), n)),
            _ => None,
        })
        .collect()
}

/// The renamed-to identifier of a `pub use path::To as Name;`, at any depth.
fn use_rename(tree: &syn::UseTree) -> Option<String> {
    match tree {
        syn::UseTree::Rename(r) => Some(r.rename.to_string()),
        syn::UseTree::Path(p) => use_rename(&p.tree),
        _ => None,
    }
}

fn schema(json: &str) -> RootSchema {
    serde_json::from_str(json).unwrap()
}

fn named(items: &[(String, String)], kind: &str, name: &str) -> bool {
    items.iter().any(|(k, n)| k == kind && n == name)
}

#[test]
fn identical_structs_collapse_to_one_definition_plus_an_alias() {
    let s = schema(
        r##"{
        "type": "object",
        "properties": {
            "a": { "$ref": "#/definitions/Alpha" },
            "b": { "$ref": "#/definitions/Beta" }
        },
        "definitions": {
            "Alpha": { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } },
            "Beta":  { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } }
        }
    }"##,
    );

    let collapsed = items(&s, TypeDedup::Collapse);
    assert!(named(&collapsed, "struct", "Alpha"), "{collapsed:?}");
    assert!(named(&collapsed, "alias", "Beta"), "{collapsed:?}");
    assert!(!named(&collapsed, "struct", "Beta"), "{collapsed:?}");

    // Opting out restores a full definition for both.
    let per_name = items(&s, TypeDedup::PerName);
    assert!(named(&per_name, "struct", "Alpha"), "{per_name:?}");
    assert!(named(&per_name, "struct", "Beta"), "{per_name:?}");
    assert!(!named(&per_name, "alias", "Beta"), "{per_name:?}");
}

#[test]
fn a_differing_description_still_collapses_because_the_alias_carries_it() {
    let s = schema(
        r##"{
        "type": "object",
        "properties": {
            "a": { "$ref": "#/definitions/Alpha" },
            "b": { "$ref": "#/definitions/Beta" }
        },
        "definitions": {
            "Alpha": { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } },
            "Beta":  { "description": "Same shape, its own docs.",
                       "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } }
        }
    }"##,
    );

    let collapsed = items(&s, TypeDedup::Collapse);
    assert!(named(&collapsed, "alias", "Beta"), "{collapsed:?}");

    let mut settings = TypeSpaceSettings::default();
    settings.with_type_dedup(TypeDedup::Collapse);
    let mut type_space = TypeSpace::new(&settings);
    type_space.add_root_schema(s).unwrap();
    let text = type_space.to_stream().to_string();
    assert!(
        text.contains("Same shape, its own docs."),
        "the alias must keep the description it replaced"
    );
}

#[test]
fn a_differing_field_type_does_not_collapse() {
    let s = schema(
        r##"{
        "type": "object",
        "properties": {
            "a": { "$ref": "#/definitions/Alpha" },
            "c": { "$ref": "#/definitions/Gamma" }
        },
        "definitions": {
            "Alpha": { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } },
            "Gamma": { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "integer" } } }
        }
    }"##,
    );

    let collapsed = items(&s, TypeDedup::Collapse);
    assert!(named(&collapsed, "struct", "Alpha"), "{collapsed:?}");
    assert!(named(&collapsed, "struct", "Gamma"), "{collapsed:?}");
    assert!(!named(&collapsed, "alias", "Gamma"), "{collapsed:?}");
}

#[test]
fn newtypes_that_differ_only_in_a_constraint_do_not_collapse() {
    // `maxLength` lives in a field whose `Ord` compares equal to everything, so
    // an ordering-based dedup key would merge these and silently widen `Narrow`.
    let s = schema(
        r##"{
        "type": "object",
        "properties": {
            "d": { "$ref": "#/definitions/Narrow" },
            "e": { "$ref": "#/definitions/Wide" }
        },
        "definitions": {
            "Narrow": { "type": "string", "maxLength": 4 },
            "Wide":   { "type": "string", "maxLength": 64 }
        }
    }"##,
    );

    let collapsed = items(&s, TypeDedup::Collapse);
    assert!(named(&collapsed, "struct", "Narrow"), "{collapsed:?}");
    assert!(named(&collapsed, "struct", "Wide"), "{collapsed:?}");
    assert!(!named(&collapsed, "alias", "Wide"), "{collapsed:?}");
}

#[test]
fn enums_that_differ_only_in_their_permitted_values_do_not_collapse() {
    // Same shape as above: the permitted values are held in a field that orders
    // as equal, so only an equality-based key keeps these apart.
    let s = schema(
        r##"{
        "type": "object",
        "properties": {
            "p": { "$ref": "#/definitions/Pair" },
            "t": { "$ref": "#/definitions/Trio" }
        },
        "definitions": {
            "Pair": { "type": "string", "enum": ["one", "two"] },
            "Trio": { "type": "string", "enum": ["one", "two", "three"] }
        }
    }"##,
    );

    let collapsed = items(&s, TypeDedup::Collapse);
    assert!(named(&collapsed, "enum", "Pair"), "{collapsed:?}");
    assert!(named(&collapsed, "enum", "Trio"), "{collapsed:?}");
    assert!(!named(&collapsed, "alias", "Trio"), "{collapsed:?}");
}

#[test]
fn parents_collapse_once_their_children_have() {
    // ParentOne holds an Alpha and ParentTwo a Beta. They are only
    // interchangeable after Alpha and Beta merge, which takes a second round.
    let s = schema(
        r##"{
        "type": "object",
        "properties": {
            "f": { "$ref": "#/definitions/ParentOne" },
            "g": { "$ref": "#/definitions/ParentTwo" }
        },
        "definitions": {
            "Alpha": { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } },
            "Beta":  { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } },
            "ParentOne": { "type": "object", "required": ["kid"],
                           "properties": { "kid": { "$ref": "#/definitions/Alpha" } } },
            "ParentTwo": { "type": "object", "required": ["kid"],
                           "properties": { "kid": { "$ref": "#/definitions/Beta" } } }
        }
    }"##,
    );

    let collapsed = items(&s, TypeDedup::Collapse);
    assert!(named(&collapsed, "struct", "ParentOne"), "{collapsed:?}");
    assert!(named(&collapsed, "alias", "ParentTwo"), "{collapsed:?}");
    assert!(!named(&collapsed, "struct", "ParentTwo"), "{collapsed:?}");
}

#[test]
fn collapsed_structs_keep_a_builder_of_their_own_name() {
    let s = schema(
        r##"{
        "type": "object",
        "properties": {
            "a": { "$ref": "#/definitions/Alpha" },
            "b": { "$ref": "#/definitions/Beta" }
        },
        "definitions": {
            "Alpha": { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } },
            "Beta":  { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } }
        }
    }"##,
    );

    let mut settings = TypeSpaceSettings::default();
    settings.with_struct_builder(true);
    let mut type_space = TypeSpace::new(&settings);
    type_space.add_root_schema(s).unwrap();
    let file = syn::parse2::<syn::File>(type_space.to_stream()).unwrap();

    let builder = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Mod(m) if m.ident == "builder" => m.content.as_ref(),
            _ => None,
        })
        .expect("a builder module");
    let aliased = builder.1.iter().any(|item| match item {
        syn::Item::Use(u) => use_rename(&u.tree).as_deref() == Some("Beta"),
        _ => false,
    });
    assert!(aliased, "builder::Beta must resolve after Beta collapses");
}

#[test]
fn generated_code_parses_and_every_original_name_is_still_declared() {
    let s = schema(
        r##"{
        "type": "object",
        "properties": {
            "a": { "$ref": "#/definitions/Alpha" },
            "b": { "$ref": "#/definitions/Beta" },
            "c": { "$ref": "#/definitions/Gamma" }
        },
        "definitions": {
            "Alpha": { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } },
            "Beta":  { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "string" } } },
            "Gamma": { "type": "object", "required": ["x"],
                       "properties": { "x": { "type": "integer" } } }
        }
    }"##,
    );

    let collapsed = items(&s, TypeDedup::Collapse);
    for name in ["Alpha", "Beta", "Gamma"] {
        assert!(
            collapsed.iter().any(|(_, n)| n == name),
            "{name} must still be declared, as a definition or an alias: {collapsed:?}"
        );
    }
}

#[test]
fn identical_string_enums_collapse() {
    let s = schema(
        r##"{
        "type": "object",
        "properties": {
            "p": { "$ref": "#/definitions/One" },
            "q": { "$ref": "#/definitions/Two" }
        },
        "definitions": {
            "One": { "type": "string", "enum": ["alpha", "beta"] },
            "Two": { "type": "string", "enum": ["alpha", "beta"] }
        }
    }"##,
    );

    let collapsed = items(&s, TypeDedup::Collapse);
    assert!(named(&collapsed, "enum", "One"), "{collapsed:?}");
    assert!(named(&collapsed, "alias", "Two"), "{collapsed:?}");
}

/// Every `impl From<Src> for Dst` in the generated code, as rendered tokens.
fn from_impls(schema: &RootSchema, dedup: TypeDedup) -> Vec<String> {
    use quote::ToTokens;

    let mut settings = TypeSpaceSettings::default();
    settings.with_type_dedup(dedup);
    let mut type_space = TypeSpace::new(&settings);
    type_space.add_root_schema(schema.clone()).unwrap();
    let file = syn::parse2::<syn::File>(type_space.to_stream()).unwrap();

    fn walk(items: &[syn::Item], out: &mut Vec<String>) {
        for item in items {
            match item {
                syn::Item::Mod(m) => {
                    if let Some((_, inner)) = &m.content {
                        walk(inner, out);
                    }
                }
                syn::Item::Impl(i) => {
                    if let Some((_, path, _)) = &i.trait_ {
                        let rendered = path.to_token_stream().to_string();
                        if rendered.contains("From <") || rendered.contains("From<") {
                            out.push(format!("{} for {}", rendered, i.self_ty.to_token_stream()));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut out = Vec::new();
    walk(&file.items, &mut out);
    out
}

#[test]
fn collapsing_never_produces_two_from_impls_with_the_same_source() {
    // An enum emits `impl From<Payload> for Enum` per variant. Two variants
    // whose payloads collapsed onto one type would be a duplicate impl, so such
    // payloads have to keep their own definitions.
    let s = schema(
        r##"{
        "type": "object",
        "properties": { "u": { "$ref": "#/definitions/Union" } },
        "definitions": {
            "Left":  { "type": "object", "required": ["a"], "additionalProperties": false,
                       "properties": { "a": { "type": "string" } } },
            "Right": { "type": "object", "required": ["a"], "additionalProperties": false,
                       "properties": { "a": { "type": "string" } } },
            "Union": { "oneOf": [ { "$ref": "#/definitions/Left" },
                                  { "$ref": "#/definitions/Right" } ] }
        }
    }"##,
    );

    let impls = from_impls(&s, TypeDedup::Collapse);
    let distinct = impls.iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        impls.len(),
        distinct.len(),
        "duplicate From impls would not compile: {impls:?}"
    );
}
