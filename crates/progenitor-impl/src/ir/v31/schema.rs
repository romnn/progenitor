// Copyright 2026 Oxide Computer Company

//! JSON Schema 2020-12 → draft-07 lowering for the OpenAPI 3.1 frontend.
//!
//! OpenAPI 3.1 schemas are full JSON Schema 2020-12, while typify consumes
//! the schemars 0.8 draft-07 model. This module rewrites schema values
//! between the dialects as a pure `Value → Value` transformation;
//! deserialization into `schemars` happens as a separate final step
//! ([`into_schemars`]) so the rewrite logic survives any future change of
//! the generator's schema representation.
//!
//! Where a 2020-12 construct has a 3.0 equivalent, the rewrite targets the
//! exact shapes `to_schema.rs` produces for that equivalent — typify embeds
//! the schemars value in generated doc comments, so a 3.0 document and its
//! 3.1 twin only generate identical code if their lowered schemas are
//! byte-equal.

use indexmap::IndexMap;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

use crate::{Error, Result};

/// Document-wide schema lowering state: the set of claimed component
/// names, schemas hoisted out of `$defs`, and the `$ref` rewrites those
/// hoists imply.
pub(super) struct SchemaLowering {
    /// Every claimed component name (original components + hoisted defs);
    /// used both for unique-name synthesis and `$ref` validation. typify
    /// resolves references by their final path segment, so names must be
    /// unique in that segment.
    known: indexmap::IndexSet<String>,
    /// Hoisted `$defs` members in discovery order: (name, schema value,
    /// the tree-local `$defs` name map for resolving its own short-form
    /// refs).
    hoisted: Vec<(String, Value, BTreeMap<String, String>)>,
    /// Full JSON-pointer `$ref` → rewritten `$ref`, for refs that point
    /// into a `$defs` member by its original document location.
    full_renames: BTreeMap<String, String>,
    /// Schemas hoisted by the most recent [`Self::lower_inline`] call,
    /// awaiting collection via [`Self::take_pending_hoists`].
    pending_hoists: Vec<(String, schemars::schema::Schema)>,
}

impl SchemaLowering {
    pub(super) fn new(component_names: impl IntoIterator<Item = String>) -> Self {
        Self {
            known: component_names.into_iter().collect(),
            hoisted: Vec::new(),
            full_renames: BTreeMap::new(),
            pending_hoists: Vec::new(),
        }
    }

    /// Lower the `components.schemas` map: hoist every nested `$defs`
    /// member into a component of its own, then apply the dialect rewrite
    /// to originals and hoists alike. Returns the final component map in
    /// document order with hoisted schemas appended in discovery order.
    pub(super) fn lower_components(
        &mut self,
        schemas: IndexMap<String, Value>,
    ) -> Result<IndexMap<String, schemars::schema::Schema>> {
        // Hoist before any rewriting: refs may point across schemas into
        // another component's `$defs`, so the full rename map must be
        // complete first.
        let mut prepared = Vec::new();
        for (name, mut value) in schemas {
            let mut local = BTreeMap::new();
            let owner = format!("#/components/schemas/{name}");
            self.hoist_defs(&mut value, &owner, &mut local)?;
            prepared.push((name, value, local));
        }

        let mut out = IndexMap::new();
        for (name, mut value, local) in prepared {
            self.rewrite(&mut value, &local)?;
            out.insert(name.clone(), into_schemars(&value, &name)?);
        }
        // Hoisted entries may themselves hoist further `$defs`, growing
        // the list while we iterate.
        let mut index = 0;
        while index < self.hoisted.len() {
            let (name, mut value, local) = self.hoisted[index].clone();
            self.rewrite(&mut value, &local)?;
            out.insert(name.clone(), into_schemars(&value, &name)?);
            index += 1;
        }
        Ok(out)
    }

    /// Lower a single inline schema (parameter, request/response body).
    /// Must be called after [`Self::lower_components`] so cross-references
    /// resolve. `context` names the schema's location for error messages.
    pub(super) fn lower_inline(
        &mut self,
        mut value: Value,
        context: &str,
    ) -> Result<schemars::schema::Schema> {
        let mut local = BTreeMap::new();
        let owner = format!("#/{context}");
        self.hoist_defs(&mut value, &owner, &mut local)?;
        self.rewrite(&mut value, &local)?;
        // Process any defs the inline schema hoisted.
        let mut extra = Vec::new();
        let mut index = 0;
        while index < self.hoisted.len() {
            let (name, mut hoisted_value, hoisted_local) = self.hoisted[index].clone();
            self.rewrite(&mut hoisted_value, &hoisted_local)?;
            extra.push((name.clone(), into_schemars(&hoisted_value, &name)?));
            index += 1;
        }
        self.hoisted.clear();
        self.pending_hoists = extra;
        into_schemars(&value, context)
    }

    /// Take schemas hoisted by the most recent [`Self::lower_inline`] call
    /// so the caller can append them to the document's component map.
    pub(super) fn take_pending_hoists(&mut self) -> Vec<(String, schemars::schema::Schema)> {
        std::mem::take(&mut self.pending_hoists)
    }

    fn unique_name(&mut self, key: &str, owner_hint: &str) -> String {
        let mut candidate = key.to_string();
        if self.known.contains(&candidate) {
            candidate = format!("{owner_hint}{key}");
        }
        let mut counter = 2;
        while self.known.contains(&candidate) {
            candidate = format!("{owner_hint}{key}{counter}");
            counter += 1;
        }
        self.known.insert(candidate.clone());
        candidate
    }

    /// Recursively remove `$defs` maps, registering each member as a
    /// to-be-hoisted component schema. `pointer` tracks the value's
    /// original document location (for rewriting full-pointer refs);
    /// `local` collects `def key → hoisted name` for the surrounding
    /// schema tree (for rewriting `#/$defs/<key>` shorthand refs).
    fn hoist_defs(
        &mut self,
        value: &mut Value,
        pointer: &str,
        local: &mut BTreeMap<String, String>,
    ) -> Result<()> {
        let Value::Object(map) = value else {
            return Ok(());
        };

        if let Some(defs) = map.shift_remove("$defs") {
            let Value::Object(defs) = defs else {
                return Err(Error::UnexpectedFormat(format!(
                    "{pointer}/$defs is not an object"
                )));
            };
            let owner_hint = pointer.rsplit('/').next().unwrap_or_default().to_string();
            for (key, mut def) in defs {
                let name = self.unique_name(&key, &owner_hint);
                self.full_renames.insert(
                    format!("{pointer}/$defs/{key}"),
                    format!("#/components/schemas/{name}"),
                );
                local.insert(key.clone(), name.clone());

                // The def's own subtree resolves shorthand refs against
                // its own defs first, then the enclosing tree's.
                let mut def_local = local.clone();
                let def_pointer = format!("{pointer}/$defs/{key}");
                self.hoist_defs(&mut def, &def_pointer, &mut def_local)?;
                self.hoisted.push((name, def, def_local));
            }
        }

        for (key, child) in map.iter_mut() {
            let key = key.clone();
            match child {
                Value::Object(_) if recurses_into_object(&key) => {
                    self.hoist_defs(child, &format!("{pointer}/{key}"), local)?;
                }
                Value::Object(child_map) if recurses_into_map_values(&key) => {
                    for (prop, prop_value) in child_map.iter_mut() {
                        let prop_pointer = format!("{pointer}/{key}/{}", escape_pointer(prop));
                        self.hoist_defs(prop_value, &prop_pointer, local)?;
                    }
                }
                Value::Array(items) if recurses_into_array(&key) => {
                    for (idx, item) in items.iter_mut().enumerate() {
                        self.hoist_defs(item, &format!("{pointer}/{key}/{idx}"), local)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Apply the 2020-12 → draft-07 keyword rewrites to a schema value,
    /// recursively.
    fn rewrite(&self, value: &mut Value, local: &BTreeMap<String, String>) -> Result<()> {
        match value {
            // Boolean schemas are native to both dialects.
            Value::Bool(_) => Ok(()),
            Value::Object(_) => self.rewrite_object(value, local),
            _ => Ok(()),
        }
    }

    fn rewrite_object(&self, value: &mut Value, local: &BTreeMap<String, String>) -> Result<()> {
        let Value::Object(map) = value else {
            return Ok(());
        };

        for unsupported in ["$dynamicRef", "$dynamicAnchor"] {
            if map.contains_key(unsupported) {
                return Err(Error::UnexpectedFormat(format!(
                    "the JSON Schema keyword `{unsupported}` is not supported"
                )));
            }
        }
        // `$anchor` only matters as a `$ref` target; refs to anchors fail
        // ref validation with a clear error, so the keyword itself can go.
        map.shift_remove("$anchor");

        // OpenAPI's `discriminator` is not a JSON Schema keyword; carry it
        // as the `x-discriminator` extension exactly like the 3.0 frontend
        // so typify's discriminator prepass sees one spelling.
        if let Some(discriminator) = map.shift_remove("discriminator") {
            map.entry("x-discriminator").or_insert(discriminator);
        }

        // `example` (singular) is the deprecated 3.0 spelling; schemars
        // models the draft-07 `examples` array, which is also what the
        // 3.0 frontend produces.
        if let Some(example) = map.shift_remove("example")
            && !map.contains_key("examples")
        {
            map.insert("examples".to_string(), Value::Array(vec![example]));
        }
        if let Some(Value::Object(examples)) = map.get("examples") {
            let values = examples
                .values()
                .map(|example| {
                    example
                        .get("value")
                        .cloned()
                        .unwrap_or_else(|| example.clone())
                })
                .collect();
            map.insert("examples".to_string(), Value::Array(values));
        }

        if matches!(map.get("properties"), Some(Value::Null)) {
            map.shift_remove("properties");
        }

        // typify ignores `const` entirely; a single-value `enum` is
        // equivalent and matches what 3.0 documents express.
        if let Some(constant) = map.shift_remove("const") {
            map.entry("enum")
                .or_insert_with(|| Value::Array(vec![constant]));
        }

        // Draft-4-style boolean exclusive bounds appear in mechanically
        // converted 3.1 documents; fold them into the numeric draft-07
        // form the same way the 3.0 frontend does.
        for (exclusive_key, bound_key) in [
            ("exclusiveMinimum", "minimum"),
            ("exclusiveMaximum", "maximum"),
        ] {
            match map.get(exclusive_key) {
                Some(Value::Bool(true)) => {
                    map.shift_remove(exclusive_key);
                    if let Some(bound) = map.shift_remove(bound_key) {
                        map.insert(exclusive_key.to_string(), bound);
                    }
                }
                Some(Value::Bool(false)) => {
                    map.shift_remove(exclusive_key);
                }
                _ => {}
            }
        }

        // typify's schema merge has no conditional support (it panics);
        // for a client generator, dropping validation-only conditionals
        // is the correct permissive degrade.
        for conditional in ["if", "then", "else"] {
            map.shift_remove(conditional);
        }

        self.rewrite_prefix_items(map);
        self.rewrite_items_array(map);

        // Fold the hybrid `nullable: true` (3.0 spelling appearing in
        // wild 3.1 documents) into the type set / nullable wrapper.
        let nullable = matches!(map.get("nullable"), Some(Value::Bool(true)));
        map.shift_remove("nullable");
        if nullable && !map.contains_key("$ref") {
            if let Some(Value::String(single)) = map.get("type") {
                let single = single.clone();
                map.insert("type".to_string(), json!([single, "null"]));
            } else if !has_type_array_with_null(map)
                && ["oneOf", "anyOf", "allOf", "not"]
                    .iter()
                    .any(|key| map.contains_key(*key))
            {
                wrap_nullable(map);
            }
        }

        // `$ref` with siblings is legal in 2020-12. typify merges most
        // siblings through its def-chain merge, but a `type` sibling next
        // to a `$ref` trips an assertion — and it is redundant with the
        // referenced schema anyway. A `["T", "null"]` type sibling carries
        // real information (the ref target is nullable here); express it
        // with the same oneOf wrapper the 3.0 frontend uses for nullable
        // refs.
        if map.contains_key("$ref") {
            let had_null = has_type_array_with_null(map) || nullable;
            map.shift_remove("type");
            if had_null {
                wrap_nullable(map);
            }
        }

        self.canonicalize_type(map);
        self.rewrite_nullable_unions(map);

        if let Some(Value::String(reference)) = map.get("$ref") {
            let rewritten = self.rewrite_ref(reference, local)?;
            map.insert("$ref".to_string(), Value::String(rewritten));
        }

        // Recurse into child schema positions only — never into data
        // positions like `enum`, `default`, or `examples`, whose values
        // may coincidentally look like schemas.
        let keys: Vec<String> = map.keys().cloned().collect();
        for key in keys {
            let Some(child) = map.get_mut(&key) else {
                continue;
            };
            if recurses_into_object(&key) {
                self.rewrite(child, local)?;
            } else if recurses_into_map_values(&key) {
                if let Value::Object(children) = child {
                    for (_, child_value) in children.iter_mut() {
                        self.rewrite(child_value, local)?;
                    }
                }
            } else if recurses_into_array(&key) {
                match child {
                    Value::Array(items) => {
                        for item in items.iter_mut() {
                            self.rewrite(item, local)?;
                        }
                    }
                    // `items` may also be a single schema.
                    Value::Object(_) | Value::Bool(_) => self.rewrite(child, local)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Lower 2020-12 tuple syntax (`prefixItems`) to the draft-07 form
    /// typify understands, or degrade when typify can't express it.
    fn rewrite_prefix_items(&self, map: &mut Map<String, Value>) {
        // Validation-only array keywords with no draft-07/typify
        // counterpart.
        for stripped in ["contains", "minContains", "maxContains"] {
            map.shift_remove(stripped);
        }

        let Some(prefix) = map.shift_remove("prefixItems") else {
            return;
        };
        let Value::Array(prefix_items) = prefix else {
            return;
        };
        let len = prefix_items.len();

        let closed = matches!(map.get("items"), Some(Value::Bool(false)));
        if closed && len > 0 {
            // Closed tuple: draft-07 spells this `items: [..]` and typify
            // requires pinned bounds to emit a Rust tuple.
            map.insert("items".to_string(), Value::Array(prefix_items));
            map.entry("minItems").or_insert(json!(len));
            map.entry("maxItems").or_insert(json!(len));
        } else {
            // Open tuple (additional items allowed): typify cannot type
            // heterogeneous prefixes with an unpinned length, so fall back
            // to an untyped array rather than failing the whole document.
            map.shift_remove("items");
        }
    }

    /// Some wild 3.1 documents still emit draft-07 tuple syntax under
    /// `items`. typify can represent pinned tuples, but unpinned item
    /// arrays in API specs usually mean "any element may match any of
    /// these schemas"; lower that shape to a homogeneous `oneOf` item.
    fn rewrite_items_array(&self, map: &mut Map<String, Value>) {
        let Some(Value::Array(items)) = map.get("items").cloned() else {
            return;
        };
        if items.is_empty() || has_pinned_tuple_bounds(map, items.len()) {
            return;
        }
        map.insert("items".to_string(), json!({ "oneOf": items }));
    }

    /// Normalize `type` arrays: drop duplicates, unwrap single-element
    /// arrays, and order `"null"` last so the lowered value matches the
    /// `[T, Null]` shape the 3.0 frontend produces for `nullable: true`.
    fn canonicalize_type(&self, map: &mut Map<String, Value>) {
        let Some(Value::Array(types)) = map.get("type") else {
            return;
        };
        let mut seen = Vec::new();
        let mut has_null = false;
        for entry in types {
            let Value::String(name) = entry else {
                return;
            };
            if name == "null" {
                has_null = true;
            } else if !seen.contains(name) {
                seen.push(name.clone());
            }
        }
        let new_type = match (seen.len(), has_null) {
            (0, _) => json!("null"),
            (1, false) => json!(seen[0]),
            _ => {
                let mut all: Vec<Value> = seen.into_iter().map(Value::String).collect();
                if has_null {
                    all.push(json!("null"));
                }
                Value::Array(all)
            }
        };
        map.insert("type".to_string(), new_type);
    }

    /// typify only collapses a union to `Option<T>` when it has exactly
    /// one non-null branch. A `oneOf`/`anyOf` with two or more non-null
    /// branches plus a null branch must be restructured into the nested
    /// `oneOf: [null, <union>]` wrapper that the 3.0 frontend emits for
    /// `nullable: true` unions.
    fn rewrite_nullable_unions(&self, map: &mut Map<String, Value>) {
        for key in ["oneOf", "anyOf"] {
            let Some(Value::Array(branches)) = map.get(key) else {
                continue;
            };
            let null_count = branches.iter().filter(|b| is_null_branch(b)).count();
            let non_null: Vec<Value> = branches
                .iter()
                .filter(|b| !is_null_branch(b))
                .cloned()
                .collect();
            if null_count == 0 || non_null.len() < 2 {
                continue;
            }
            map.insert(key.to_string(), Value::Array(non_null));
            wrap_nullable(map);
        }
    }

    fn rewrite_ref(&self, reference: &str, local: &BTreeMap<String, String>) -> Result<String> {
        if let Some(renamed) = self.full_renames.get(reference) {
            return Ok(renamed.clone());
        }
        if let Some(key) = reference.strip_prefix("#/$defs/") {
            if let Some(name) = local.get(key) {
                return Ok(format!("#/components/schemas/{name}"));
            }
            return Err(Error::UnexpectedFormat(format!(
                "unresolved $defs reference: {reference}"
            )));
        }
        if let Some(name) = reference.strip_prefix("#/components/schemas/") {
            if !name.contains('/') {
                if self.known.contains(name) {
                    return Ok(reference.to_string());
                }
                return Err(Error::UnexpectedFormat(format!(
                    "unresolved schema reference: {reference}"
                )));
            }
            return Err(Error::UnexpectedFormat(format!(
                "JSON-pointer references into a schema body are not supported: {reference}"
            )));
        }
        if reference.starts_with('#') {
            return Err(Error::UnexpectedFormat(format!(
                "references to non-schema components are not supported in schema position: \
                 {reference}"
            )));
        }
        Err(Error::UnexpectedFormat(format!(
            "external references are not supported: {reference}"
        )))
    }
}

/// Convert a rewritten schema value into the schemars draft-07 model.
pub(super) fn into_schemars(value: &Value, context: &str) -> Result<schemars::schema::Schema> {
    let json = value.to_string();
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    serde_path_to_error::deserialize(&mut deserializer).map_err(|err| {
        Error::UnexpectedFormat(format!(
            "schema for {context} is not valid at {}: {}",
            err.path(),
            err.inner()
        ))
    })
}

/// Replicate `to_schema.rs`'s `oneof_nullable_wrapper`: descriptive
/// metadata and extensions stay on the outer schema; every structural
/// keyword moves into the non-null branch of `oneOf: [null, inner]`.
fn wrap_nullable(map: &mut Map<String, Value>) {
    const METADATA_KEYS: [&str; 7] = [
        "title",
        "description",
        "default",
        "deprecated",
        "readOnly",
        "writeOnly",
        "examples",
    ];
    let mut outer = Map::new();
    let mut inner = Map::new();
    for (key, value) in std::mem::take(map) {
        if METADATA_KEYS.contains(&key.as_str()) || key.starts_with("x-") {
            outer.insert(key, value);
        } else {
            inner.insert(key, value);
        }
    }
    outer.insert(
        "oneOf".to_string(),
        json!([{ "type": "null" }, Value::Object(inner)]),
    );
    *map = outer;
}

fn has_type_array_with_null(map: &Map<String, Value>) -> bool {
    matches!(
        map.get("type"),
        Some(Value::Array(types)) if types.iter().any(|t| t == "null")
    )
}

fn has_pinned_tuple_bounds(map: &Map<String, Value>, len: usize) -> bool {
    let Some(len) = u64::try_from(len).ok() else {
        return false;
    };
    matches!(
        (map.get("minItems"), map.get("maxItems")),
        (Some(Value::Number(min)), Some(Value::Number(max)))
            if min.as_u64() == Some(len) && max.as_u64() == Some(len)
    )
}

/// A union branch that exists only to admit `null`.
fn is_null_branch(value: &Value) -> bool {
    let Value::Object(map) = value else {
        return false;
    };
    matches!(map.get("type"), Some(Value::String(t)) if t == "null")
        && map
            .keys()
            .all(|k| matches!(k.as_str(), "type" | "description" | "title"))
}

/// Child keys holding a single subschema.
fn recurses_into_object(key: &str) -> bool {
    matches!(
        key,
        "additionalProperties" | "additionalItems" | "not" | "propertyNames" | "items"
    )
}

/// Child keys holding a map whose values are subschemas.
fn recurses_into_map_values(key: &str) -> bool {
    matches!(key, "properties" | "patternProperties")
}

/// Child keys holding an array of subschemas (`items` may be either form).
fn recurses_into_array(key: &str) -> bool {
    matches!(key, "allOf" | "anyOf" | "oneOf" | "items" | "prefixItems")
}

fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use serde_json::{Value, json};

    use super::SchemaLowering;
    use crate::to_schema::ToSchema;

    fn lower_components(schemas: Vec<(&str, Value)>) -> IndexMap<String, schemars::schema::Schema> {
        let schemas: IndexMap<String, Value> = schemas
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect();
        let mut lowering = SchemaLowering::new(schemas.keys().cloned());
        lowering.lower_components(schemas).unwrap()
    }

    fn lower_one(value: Value) -> schemars::schema::Schema {
        lower_components(vec![("Test", value)])
            .shift_remove("Test")
            .unwrap()
    }

    /// Lower a 3.0-dialect schema value through the v30 path
    /// (openapiv3 + to_schema), for twin-equality assertions.
    fn lower_v30(value: Value) -> schemars::schema::Schema {
        serde_json::from_value::<openapiv3::Schema>(value)
            .unwrap()
            .to_schema()
    }

    #[test]
    fn nullable_twins_lower_identically() {
        // 3.1 type array vs 3.0 nullable — must be byte-equal because
        // typify embeds the schemars value in generated doc comments.
        assert_eq!(
            lower_one(json!({"type": ["string", "null"]})),
            lower_v30(json!({"type": "string", "nullable": true})),
        );
        // Order and duplicates normalize.
        assert_eq!(
            lower_one(json!({"type": ["null", "string", "null"]})),
            lower_v30(json!({"type": "string", "nullable": true})),
        );
        // Single-element arrays unwrap.
        assert_eq!(
            lower_one(json!({"type": ["integer"]})),
            lower_v30(json!({"type": "integer"})),
        );
    }

    #[test]
    fn nullable_composite_twins_lower_identically() {
        let v31 = lower_one(json!({
            "title": "thing",
            "oneOf": [
                {"type": "object", "properties": {"a": {"type": "string"}}},
                {"type": "object", "properties": {"b": {"type": "integer"}}},
                {"type": "null"},
            ],
        }));
        let v30 = lower_v30(json!({
            "title": "thing",
            "nullable": true,
            "oneOf": [
                {"type": "object", "properties": {"a": {"type": "string"}}},
                {"type": "object", "properties": {"b": {"type": "integer"}}},
            ],
        }));
        assert_eq!(v31, v30);
    }

    #[test]
    fn const_twins_lower_identically() {
        assert_eq!(
            lower_one(json!({"type": "string", "const": "fixed"})),
            lower_v30(json!({"type": "string", "enum": ["fixed"]})),
        );
    }

    #[test]
    fn exclusive_bound_twins_lower_identically() {
        // 3.1 numeric form vs 3.0 boolean form.
        assert_eq!(
            lower_one(json!({"type": "number", "exclusiveMinimum": 1.0})),
            lower_v30(json!({
                "type": "number",
                "minimum": 1.0,
                "exclusiveMinimum": true
            })),
        );
        // Boolean form appearing in a 3.1 document is tolerated.
        assert_eq!(
            lower_one(json!({
                "type": "number",
                "maximum": 5.0,
                "exclusiveMaximum": true
            })),
            lower_v30(json!({
                "type": "number",
                "maximum": 5.0,
                "exclusiveMaximum": true
            })),
        );
    }

    #[test]
    fn example_twins_lower_identically() {
        assert_eq!(
            lower_one(json!({"type": "string", "examples": ["a"]})),
            lower_v30(json!({"type": "string", "example": "a"})),
        );
        // The deprecated singular spelling maps the same way.
        assert_eq!(
            lower_one(json!({"type": "string", "example": "a"})),
            lower_v30(json!({"type": "string", "example": "a"})),
        );
    }

    #[test]
    fn examples_map_lowers_to_values_array() {
        let lowered = lower_one(json!({
            "type": "string",
            "examples": {
                "first": {
                    "summary": "first example",
                    "value": "a"
                },
                "second": {
                    "summary": "second example",
                    "value": "b"
                }
            }
        }));
        let value = serde_json::to_value(lowered).unwrap();
        assert_eq!(value["examples"], json!(["a", "b"]));
    }

    #[test]
    fn null_properties_are_removed() {
        let lowered = lower_one(json!({
            "type": "object",
            "properties": null,
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string"
                        }
                    }
                }
            ]
        }));
        let value = serde_json::to_value(lowered).unwrap();
        assert!(value.get("properties").is_none());
    }

    #[test]
    fn unpinned_items_array_becomes_oneof_item() {
        let lowered = lower_one(json!({
            "type": "array",
            "items": [
                {
                    "type": "string"
                },
                {
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string"
                        }
                    }
                }
            ]
        }));
        let value = serde_json::to_value(lowered).unwrap();
        assert!(value["items"]["oneOf"].is_array());
    }

    #[test]
    fn pinned_items_array_stays_tuple() {
        let lowered = lower_one(json!({
            "type": "array",
            "minItems": 2,
            "maxItems": 2,
            "items": [
                {
                    "type": "string"
                },
                {
                    "type": "integer"
                }
            ]
        }));
        let value = serde_json::to_value(lowered).unwrap();
        assert!(value["items"].is_array());
    }

    #[test]
    fn discriminator_twins_lower_identically() {
        let discriminated = json!({
            "type": "object",
            "properties": {"kind": {"type": "string"}},
            "required": ["kind"],
            "discriminator": {"propertyName": "kind"},
        });
        assert_eq!(lower_one(discriminated.clone()), lower_v30(discriminated),);
    }

    #[test]
    fn defs_are_hoisted_into_components() {
        let lowered = lower_components(vec![(
            "Outer",
            json!({
                "type": "object",
                "properties": {
                    "shorthand": {"$ref": "#/$defs/Inner"},
                    "pointer": {
                        "$ref": "#/components/schemas/Outer/$defs/Inner"
                    },
                },
                "$defs": {
                    "Inner": {"type": "string"},
                },
            }),
        )]);

        assert!(lowered.contains_key("Inner"), "def must be hoisted");
        let outer = serde_json::to_value(&lowered["Outer"]).unwrap();
        assert_eq!(
            outer["properties"]["shorthand"]["$ref"],
            "#/components/schemas/Inner"
        );
        assert_eq!(
            outer["properties"]["pointer"]["$ref"],
            "#/components/schemas/Inner"
        );
    }

    #[test]
    fn hoisted_def_names_avoid_collisions() {
        let lowered = lower_components(vec![
            ("Inner", json!({"type": "integer"})),
            (
                "Outer",
                json!({
                    "type": "object",
                    "properties": {"x": {"$ref": "#/$defs/Inner"}},
                    "$defs": {"Inner": {"type": "string"}},
                }),
            ),
        ]);

        // The existing component keeps its name; the def gets a fresh one.
        assert!(lowered.contains_key("OuterInner"));
        let outer = serde_json::to_value(&lowered["Outer"]).unwrap();
        assert_eq!(
            outer["properties"]["x"]["$ref"],
            "#/components/schemas/OuterInner"
        );
    }

    #[test]
    fn nested_defs_are_hoisted() {
        let lowered = lower_components(vec![(
            "Outer",
            json!({
                "$defs": {
                    "Mid": {
                        "type": "object",
                        "properties": {"deep": {"$ref": "#/$defs/Leaf"}},
                        "$defs": {"Leaf": {"type": "boolean"}},
                    },
                },
                "type": "object",
                "properties": {"m": {"$ref": "#/$defs/Mid"}},
            }),
        )]);
        assert!(lowered.contains_key("Mid"));
        assert!(lowered.contains_key("Leaf"));
        let mid = serde_json::to_value(&lowered["Mid"]).unwrap();
        assert_eq!(
            mid["properties"]["deep"]["$ref"],
            "#/components/schemas/Leaf"
        );
    }

    #[test]
    fn closed_tuples_become_draft07_tuples() {
        let lowered = serde_json::to_value(lower_one(json!({
            "type": "array",
            "prefixItems": [{"type": "string"}, {"type": "integer"}],
            "items": false,
        })))
        .unwrap();
        assert_eq!(
            lowered["items"],
            json!([{"type": "string"}, {"type": "integer"}])
        );
        assert_eq!(lowered["minItems"], 2);
        assert_eq!(lowered["maxItems"], 2);
    }

    #[test]
    fn open_tuples_degrade_to_untyped_arrays() {
        let lowered = serde_json::to_value(lower_one(json!({
            "type": "array",
            "prefixItems": [{"type": "string"}],
            "items": {"type": "integer"},
        })))
        .unwrap();
        // typify can't express an unpinned heterogeneous prefix; the
        // degrade is an untyped array, not a generation failure.
        assert_eq!(lowered.get("prefixItems"), None);
        assert_eq!(lowered.get("items"), None);
    }

    #[test]
    fn ref_with_nullable_type_sibling_wraps() {
        let v31 = lower_components(vec![
            ("Target", json!({"type": "object"})),
            (
                "Holder",
                json!({
                    "type": "object",
                    "properties": {
                        "x": {
                            "$ref": "#/components/schemas/Target",
                            "type": ["object", "null"],
                        },
                    },
                }),
            ),
        ]);
        let holder = serde_json::to_value(&v31["Holder"]).unwrap();
        assert_eq!(
            holder["properties"]["x"]["oneOf"],
            json!([
                {"type": "null"},
                {"$ref": "#/components/schemas/Target"},
            ]),
        );
    }

    #[test]
    fn unresolved_and_external_refs_error() {
        let attempt = |schema: Value| -> crate::Result<_> {
            let schemas: IndexMap<String, Value> =
                [("Test".to_string(), schema)].into_iter().collect();
            let mut lowering = SchemaLowering::new(schemas.keys().cloned());
            lowering.lower_components(schemas)
        };

        let err = attempt(json!({"$ref": "#/components/schemas/Missing"})).unwrap_err();
        assert!(err.to_string().contains("unresolved schema reference"));

        let err = attempt(json!({"$ref": "./common.yaml#/Foo"})).unwrap_err();
        assert!(err.to_string().contains("external references"));

        let err = attempt(json!({"$dynamicRef": "#meta"})).unwrap_err();
        assert!(err.to_string().contains("$dynamicRef"));
    }

    #[test]
    fn conditionals_are_stripped() {
        let lowered = serde_json::to_value(lower_one(json!({
            "type": "object",
            "if": {"properties": {"a": {"const": 1}}},
            "then": {"required": ["b"]},
            "else": {"required": ["c"]},
        })))
        .unwrap();
        assert_eq!(lowered.get("if"), None);
        assert_eq!(lowered.get("then"), None);
        assert_eq!(lowered.get("else"), None);
    }
}
