// Copyright 2026 Oxide Computer Company

//! Version-agnostic internal model of an OpenAPI document.
//!
//! Each supported OpenAPI version gets its own frontend that parses the
//! source document with a version-appropriate AST and lowers it into this
//! model. Everything downstream — type generation, method generation, the
//! CLI and httpmock emitters — consumes only this model, so adding support
//! for a new spec version never touches generator code.
//!
//! Schemas are represented as JSON Schema draft-07 [`schemars`] values
//! because that is the input format `typify` consumes. Schema `$ref`s are
//! preserved (typify names and deduplicates types based on them), while
//! all other references (parameters, request bodies, responses) are
//! resolved during lowering.

pub(crate) mod v30;
pub(crate) mod v31;

use indexmap::IndexMap;
use std::collections::{BTreeMap, HashSet};

use crate::{Error, Result};

/// A parsed and validated OpenAPI document, ready for code generation.
///
/// Obtain one from [`crate::parse_openapi_str`] / [`crate::parse_openapi_value`],
/// which accept any supported spec version. Callers holding an
/// already-parsed document can round-trip it through
/// `parse_openapi_value(serde_json::to_value(&spec)?)`.
pub struct OpenApiDocument(pub(crate) Document);

pub(crate) struct Document {
    pub info: Info,
    /// `components.schemas`, converted to schemars, in document order.
    /// Order matters: typify derives type names from insertion order.
    pub schemas: IndexMap<String, schemars::schema::Schema>,
    /// All operations in document walk order (paths in document order,
    /// methods in the fixed get/put/post/… order within a path item).
    pub operations: Vec<Operation>,
    pub tags: Vec<Tag>,
}

pub(crate) struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
    pub terms_of_service: Option<String>,
}

pub(crate) struct Tag {
    pub name: String,
    pub description: Option<String>,
}

pub(crate) struct Operation {
    pub operation_id: Option<String>,
    pub tags: Vec<String>,
    /// Lower-case HTTP method name (`get`, `put`, …).
    pub method: String,
    /// Raw path template (`/v1/things/{id}`).
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    /// Path-item and operation parameters, merged (operation wins on name
    /// collision) and ordered by parameter name. This replicates the
    /// BTreeMap-based merge the generator historically performed.
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    /// The `default` response (if any) first, then the explicit status
    /// codes in document order.
    pub responses: Vec<Response>,
    /// Specification extensions (`x-…`), e.g. `x-dropshot-pagination`.
    pub extensions: IndexMap<String, serde_json::Value>,
}

pub(crate) struct Parameter {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub kind: ParameterKind,
    /// `None` when the parameter uses `content` instead of `schema`
    /// (unsupported by the generator, which reports it as such).
    pub schema: Option<schemars::schema::Schema>,
}

#[derive(Debug)]
pub(crate) enum ParameterKind {
    Path {
        style: PathStyle,
    },
    Query {
        style: QueryStyle,
        #[allow(dead_code)]
        allow_reserved: bool,
        #[allow(dead_code)]
        allow_empty_value: Option<bool>,
    },
    Header {
        style: HeaderStyle,
    },
    Cookie,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PathStyle {
    Simple,
    Matrix,
    Label,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum QueryStyle {
    Form,
    SpaceDelimited,
    PipeDelimited,
    DeepObject,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HeaderStyle {
    Simple,
}

pub(crate) struct RequestBody {
    pub description: Option<String>,
    /// Carried for future use; the generator currently treats every body
    /// as required (long-standing upstream TODO).
    #[allow(dead_code)]
    pub required: bool,
    /// Media type → content, in document order. Order is semantic: the
    /// generator prefers a JSON variant and otherwise takes the first.
    pub content: IndexMap<String, MediaTypeObject>,
}

pub(crate) struct MediaTypeObject {
    pub schema: Option<SchemaRef>,
    /// Whether the media type carries an `encoding` map. The generator
    /// does not support encodings and needs to know they were present.
    pub has_encoding: bool,
}

pub(crate) struct SchemaRef {
    /// The schema, converted to schemars with `$ref`s preserved.
    pub schema: schemars::schema::Schema,
    /// `Some(name)` iff the source was a direct `$ref` to
    /// `#/components/schemas/<name>`. Feeds the response `schema_name`
    /// used by the `allOf`-ancestor response collapse.
    pub ref_name: Option<String>,
}

pub(crate) struct Response {
    pub status: ResponseStatus,
    /// Response description; empty when the spec omitted it.
    pub description: String,
    /// Media type → content, in document order (first JSON-ish entry wins).
    pub content: IndexMap<String, MediaTypeObject>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResponseStatus {
    Code(u16),
    Range(u16),
    Default,
}

/// Validate invariants the generator relies on: every operation has an
/// operation ID and no operation ID is used twice.
pub(crate) fn validate(doc: &Document) -> Result<()> {
    let mut opids = HashSet::new();
    for operation in &doc.operations {
        if let Some(oid) = operation.operation_id.as_ref() {
            if !opids.insert(oid.to_string()) {
                return Err(Error::UnexpectedFormat(format!(
                    "duplicate operation ID: {}",
                    oid,
                )));
            }
        } else {
            return Err(Error::UnexpectedFormat(format!(
                "path {} is missing operation ID",
                operation.path,
            )));
        }
    }
    Ok(())
}

/// Follow `$ref` chains through the document's component schemas.
///
/// References are resolved by their final path segment, matching the
/// historical `ReferenceOrExt` behavior. Dangling references and cycles
/// stop the walk and return the last schema reached (callers treat a
/// still-referencing schema as a shape mismatch).
pub(crate) fn resolve_schema<'a>(
    schema: &'a schemars::schema::Schema,
    schemas: &'a IndexMap<String, schemars::schema::Schema>,
) -> &'a schemars::schema::Schema {
    let mut current = schema;
    let mut seen = 0u32;
    while let schemars::schema::Schema::Object(object) = current {
        let Some(reference) = &object.reference else {
            break;
        };
        let key = reference.rsplit('/').next().unwrap_or(reference);
        let Some(next) = schemas.get(key) else {
            break;
        };
        current = next;
        // Bound the walk so reference cycles can't hang generation.
        seen += 1;
        if seen > 64 {
            break;
        }
    }
    current
}

/// Build a `child_name → parent_name` map for every schema that extends
/// another via a top-level `allOf: [{$ref: <parent>}, ...]`.
///
/// Only schemas whose top-level `allOf` contains exactly one `$ref` to a
/// component qualify — patterns with multiple parent refs, no parent ref,
/// or non-allOf compositions don't have unambiguous "single parent"
/// semantics, so they're omitted. The map is used by
/// `find_common_supertype` to detect sibling response schemas that share
/// a common ancestor.
pub(crate) fn build_schema_supertype_map(
    schemas: &IndexMap<String, schemars::schema::Schema>,
) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (name, schema) in schemas {
        let Some(all_of) = schema_all_of(schema) else {
            continue;
        };
        let mut parent_refs: Vec<&str> = Vec::new();
        for member in all_of {
            if let schemars::schema::Schema::Object(object) = member
                && let Some(parent) = object
                    .reference
                    .as_deref()
                    .and_then(|r| r.strip_prefix("#/components/schemas/"))
            {
                parent_refs.push(parent);
            }
        }
        if parent_refs.len() == 1 {
            map.insert(name.clone(), parent_refs[0].to_string());
        }
    }
    map
}

/// Get a schema's top-level `allOf` members, looking through the
/// `oneOf: [null, …]` wrapper that the 3.0 frontend produces for
/// `nullable: true` composite schemas.
///
/// Only *pure* composition qualifies: schemas that also carry
/// type-forming keywords (a `type`, `enum`, validation constraints, …)
/// don't have plain "extends parent" semantics and are excluded, matching
/// the openapiv3 `SchemaKind::AllOf`-only behavior this map historically
/// had.
fn schema_all_of(schema: &schemars::schema::Schema) -> Option<&[schemars::schema::Schema]> {
    let schemars::schema::Schema::Object(object) = schema else {
        return None;
    };
    if !is_pure_composition(object) {
        return None;
    }
    let subschemas = object.subschemas.as_deref()?;
    if subschemas.any_of.is_some()
        || subschemas.not.is_some()
        || subschemas.if_schema.is_some()
        || subschemas.then_schema.is_some()
        || subschemas.else_schema.is_some()
    {
        return None;
    }
    match (&subschemas.all_of, subschemas.one_of.as_deref()) {
        (Some(all_of), None) => Some(all_of),
        // nullable wrapper: oneOf of exactly [null-typed schema, inner].
        (None, Some([first, inner])) if is_null_schema(first) => schema_all_of(inner),
        _ => None,
    }
}

/// Whether a schema object carries no type-forming keywords besides its
/// subschemas (metadata and extensions are fine).
fn is_pure_composition(object: &schemars::schema::SchemaObject) -> bool {
    object.instance_type.is_none()
        && object.format.is_none()
        && object.enum_values.is_none()
        && object.const_value.is_none()
        && object.reference.is_none()
        && object.string.is_none()
        && object.number.is_none()
        && object.object.is_none()
        && object.array.is_none()
}

fn is_null_schema(schema: &schemars::schema::Schema) -> bool {
    let schemars::schema::Schema::Object(object) = schema else {
        return false;
    };
    matches!(
        &object.instance_type,
        Some(schemars::schema::SingleOrVec::Single(single))
            if **single == schemars::schema::InstanceType::Null
    ) && object.subschemas.is_none()
        && object.enum_values.is_none()
        && object.reference.is_none()
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use indoc::indoc;

    use super::build_schema_supertype_map;
    use crate::to_schema::ToSchema;

    fn schemas_from_yaml(yaml: &str) -> IndexMap<String, schemars::schema::Schema> {
        let components: openapiv3::Components = serde_yaml::from_str(yaml).unwrap();
        components
            .schemas
            .iter()
            .map(|(name, schema)| (name.clone(), schema.to_schema()))
            .collect()
    }

    #[test]
    fn build_schema_supertype_map_captures_single_ref_allof() {
        let schemas = schemas_from_yaml(indoc! {"
            schemas:
              Problem:
                type: object
                properties:
                  detail: { type: string }
              BadRequestProblem:
                allOf:
                  - $ref: '#/components/schemas/Problem'
                  - type: object
                    properties:
                      violations: { type: object }
        "});
        let map = build_schema_supertype_map(&schemas);
        assert_eq!(map.get("BadRequestProblem"), Some(&"Problem".to_string()));
        // The plain `Problem` schema has no allOf and shouldn't appear.
        assert!(!map.contains_key("Problem"));
    }

    #[test]
    fn build_schema_supertype_map_skips_multiref_or_mixed_allof() {
        // Two parent refs — not a single-parent "extends" pattern.
        let schemas = schemas_from_yaml(indoc! {"
            schemas:
              ParentA:
                type: object
              ParentB:
                type: object
              Multi:
                allOf:
                  - $ref: '#/components/schemas/ParentA'
                  - $ref: '#/components/schemas/ParentB'
        "});
        let map = build_schema_supertype_map(&schemas);
        assert!(
            !map.contains_key("Multi"),
            "multi-parent allOf must not yield a single parent"
        );
    }

    #[test]
    fn build_schema_supertype_map_requires_pure_composition() {
        // An allOf that also carries type-forming keywords (here:
        // `type: object` + `properties`, openapiv3's `SchemaKind::Any`)
        // is not a plain "extends parent" pattern and must be excluded —
        // matching the historical SchemaKind::AllOf-only behavior.
        let schemas = schemas_from_yaml(indoc! {"
            schemas:
              Problem:
                type: object
              Mixed:
                type: object
                properties:
                  extra: { type: string }
                allOf:
                  - $ref: '#/components/schemas/Problem'
        "});
        let map = build_schema_supertype_map(&schemas);
        assert!(
            !map.contains_key("Mixed"),
            "allOf alongside type-forming keywords must not register a supertype"
        );
    }

    #[test]
    fn build_schema_supertype_map_sees_through_nullable_wrapper() {
        // `nullable: true` on an allOf schema makes the 3.0 frontend wrap
        // it in oneOf [null, allOf]; the supertype scan must look through.
        let schemas = schemas_from_yaml(indoc! {"
            schemas:
              Problem:
                type: object
              NullableChild:
                nullable: true
                allOf:
                  - $ref: '#/components/schemas/Problem'
        "});
        let map = build_schema_supertype_map(&schemas);
        assert_eq!(map.get("NullableChild"), Some(&"Problem".to_string()));
    }
}
