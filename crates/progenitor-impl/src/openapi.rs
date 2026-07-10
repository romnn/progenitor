use openapiv3::OpenAPI;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::ir::OpenApiDocument;

mod ref_repair;

const OPENAPI_VERSION_KEY: &str = "openapi";
const NULL_TYPE_NAME: &str = "null";
const TYPE_KEY: &str = "type";
const NULLABLE_KEY: &str = "nullable";

/// YAML deserialization tolerant of real-world spec sloppiness that the
/// straight `serde_json::Value` target rejects:
///
/// - integers beyond the i64/u64 range (JavaScript artifacts like
///   `18446744073709552000`, u64::MAX rounded through a float) fold to
///   `f64`, matching how the JSON parser and the tools that produced them
///   treat such numbers;
/// - non-string mapping keys (unquoted YAML response codes like `200:`)
///   are stringified.
mod tolerant {
    use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
    use serde_json::{Map, Value};

    pub(super) struct TolerantValue(pub Value);

    struct ValueVisitor;

    impl<'de> Visitor<'de> for ValueVisitor {
        type Value = TolerantValue;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("any YAML value")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
            Ok(TolerantValue(Value::Bool(v)))
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(TolerantValue(Value::from(v)))
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(TolerantValue(Value::from(v)))
        }
        fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E> {
            Ok(TolerantValue(Value::from(v as f64)))
        }
        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E> {
            Ok(TolerantValue(Value::from(v as f64)))
        }
        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
            Ok(TolerantValue(
                serde_json::Number::from_f64(v).map_or(Value::Null, Value::Number),
            ))
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(TolerantValue(Value::String(v.to_string())))
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(TolerantValue(Value::String(v)))
        }
        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(TolerantValue(Value::Null))
        }
        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(TolerantValue(Value::Null))
        }
        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            d.deserialize_any(self)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut items = Vec::new();
            while let Some(TolerantValue(item)) = seq.next_element()? {
                items.push(item);
            }
            Ok(TolerantValue(Value::Array(items)))
        }

        fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
            let mut map = Map::new();
            while let Some((TolerantKey(key), TolerantValue(value))) = access.next_entry()? {
                map.insert(key, value);
            }
            Ok(TolerantValue(Value::Object(map)))
        }
    }

    impl<'de> Deserialize<'de> for TolerantValue {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            d.deserialize_any(ValueVisitor)
        }
    }

    struct TolerantKey(String);

    struct KeyVisitor;

    impl Visitor<'_> for KeyVisitor {
        type Value = TolerantKey;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a scalar mapping key")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(TolerantKey(v.to_string()))
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(TolerantKey(v))
        }
        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
            Ok(TolerantKey(v.to_string()))
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(TolerantKey(v.to_string()))
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(TolerantKey(v.to_string()))
        }
        fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E> {
            Ok(TolerantKey(v.to_string()))
        }
        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E> {
            Ok(TolerantKey(v.to_string()))
        }
        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
            Ok(TolerantKey(v.to_string()))
        }
    }

    impl<'de> Deserialize<'de> for TolerantKey {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            d.deserialize_any(KeyVisitor)
        }
    }
}

/// Errors returned while normalizing and decoding an OpenAPI document.
#[derive(Debug, Error)]
pub enum ParseOpenApiError {
    /// Neither JSON nor YAML parsing succeeded for the source document.
    #[error("parse openapi document as json or yaml: json {json_message}; yaml {yaml_message}")]
    Format {
        /// The JSON parser failure message.
        json_message: String,
        /// The YAML parser failure message.
        yaml_message: String,
    },
    /// Serializing the normalized serde value failed unexpectedly.
    #[error("serialize normalized openapi document: {message}")]
    Serialize {
        /// The underlying serialization error message.
        message: String,
    },
    /// Deserializing into the version-specific AST failed at a specific path.
    #[error("decode openapi at {path}: {message}")]
    Deserialize {
        /// The serde path within the normalized document.
        path: String,
        /// The underlying deserialization error message.
        message: String,
    },
    /// The document parsed but violates an invariant the generator relies
    /// on (unsupported version, missing/duplicate operation IDs, …).
    #[error("invalid openapi document: {0}")]
    Invalid(#[from] crate::Error),
}

/// Parse a JSON or YAML OpenAPI document of any supported spec version
/// into progenitor's internal model.
pub fn parse_openapi_str(
    document: &str,
) -> std::result::Result<OpenApiDocument, ParseOpenApiError> {
    let value = match serde_json::from_str(document) {
        Ok(value) => value,
        Err(json_err) => {
            serde_yaml::from_str::<tolerant::TolerantValue>(document)
                .map_err(|yaml_err| ParseOpenApiError::Format {
                    json_message: json_err.to_string(),
                    yaml_message: yaml_err.to_string(),
                })?
                .0
        }
    };

    parse_openapi_value(value)
}

/// Parse an already-decoded OpenAPI document of any supported spec version
/// into progenitor's internal model.
pub fn parse_openapi_value(
    mut value: Value,
) -> std::result::Result<OpenApiDocument, ParseOpenApiError> {
    // Repair malformed `$ref`s on the raw value so both version frontends
    // benefit. Kind mismatches first: a misfiled component's body may
    // contain deep pointers that the hoisting pass must then see in its
    // relocated copy as well.
    ref_repair::relocate_kind_mismatched_component_refs(&mut value);
    ref_repair::hoist_deep_pointer_refs(&mut value);
    normalize_real_world_sloppiness(&mut value);

    let version = value
        .get(OPENAPI_VERSION_KEY)
        .and_then(Value::as_str)
        .unwrap_or_default();
    if version.trim().starts_with("3.1") {
        let document = crate::ir::v31::parse(value)?;
        Ok(OpenApiDocument(document))
    } else {
        let spec = normalized_openapiv3(value)?;
        let document = crate::ir::v30::lower(&spec)?;
        Ok(OpenApiDocument(document))
    }
}

/// Normalize a serde value into the shape `openapiv3` represents directly,
/// then deserialize it. The nullable-union normalization is kept on the
/// 3.0 path as tolerance for hybrid documents that mix 3.1 idioms into a
/// 3.0 version stamp.
fn normalized_openapiv3(mut value: Value) -> std::result::Result<OpenAPI, ParseOpenApiError> {
    strip_null_path_entries(&mut value);
    normalize_nullable_type_unions(&mut value);

    let json = serde_json::to_vec(&value).map_err(|err| ParseOpenApiError::Serialize {
        message: err.to_string(),
    })?;
    let mut deserializer = serde_json::Deserializer::from_slice(&json);
    serde_path_to_error::deserialize(&mut deserializer).map_err(|err| {
        ParseOpenApiError::Deserialize {
            path: err.path().to_string(),
            message: err.inner().to_string(),
        }
    })
}

fn normalize_real_world_sloppiness(value: &mut Value) {
    ensure_info_version(value);
    normalize_type_strings(value);
    normalize_response_descriptions(value);
    normalize_non_query_deep_object_parameters(value);
}

fn ensure_info_version(value: &mut Value) {
    let Some(info) = value.get_mut("info").and_then(Value::as_object_mut) else {
        return;
    };
    if !matches!(info.get("version"), Some(Value::String(_))) {
        info.insert("version".to_string(), Value::String("unknown".to_string()));
    }
}

fn normalize_type_strings(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(type_name)) = map.get(TYPE_KEY) {
                let canonical = type_name.to_ascii_lowercase();
                if type_name.is_empty() || canonical == "unknown" {
                    map.shift_remove(TYPE_KEY);
                } else if matches!(
                    canonical.as_str(),
                    "null" | "boolean" | "object" | "array" | "number" | "string" | "integer"
                ) && canonical != *type_name
                {
                    map.insert(TYPE_KEY.to_string(), Value::String(canonical));
                }
            }
            for entry in map.values_mut() {
                normalize_type_strings(entry);
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize_type_strings(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn normalize_response_descriptions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Object(responses)) = map.get_mut("responses") {
                for response in responses.values_mut() {
                    let Some(response) = response.as_object_mut() else {
                        continue;
                    };
                    if response.contains_key("$ref") {
                        continue;
                    }
                    if !matches!(response.get("description"), Some(Value::String(_))) {
                        response.insert("description".to_string(), Value::String(String::new()));
                    }
                }
            }
            for entry in map.values_mut() {
                normalize_response_descriptions(entry);
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize_response_descriptions(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn normalize_non_query_deep_object_parameters(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let is_non_query_deep_object = matches!(
                (map.get("in"), map.get("style")),
                (Some(Value::String(location)), Some(Value::String(style)))
                    if location != "query" && style == "deepObject"
            );
            if is_non_query_deep_object {
                map.shift_remove("style");
            }
            for entry in map.values_mut() {
                normalize_non_query_deep_object_parameters(entry);
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize_non_query_deep_object_parameters(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn normalize_nullable_type_unions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for entry in map.values_mut() {
                normalize_nullable_type_unions(entry);
            }
            normalize_object_type_union(map);
            normalize_draft4_exclusive_bounds(map);
        }
        Value::Array(items) => {
            for item in items {
                normalize_nullable_type_unions(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

/// 3.0 documents are supposed to spell exclusive bounds as booleans
/// modifying `minimum`/`maximum`, but generators that think in JSON
/// Schema 2020-12 emit the numeric form into 3.0 documents anyway.
/// Fold the numeric form back into the boolean spelling `openapiv3`
/// expects.
fn normalize_draft4_exclusive_bounds(map: &mut Map<String, Value>) {
    for (exclusive_key, bound_key) in [
        ("exclusiveMinimum", "minimum"),
        ("exclusiveMaximum", "maximum"),
    ] {
        if let Some(Value::Number(bound)) = map.get(exclusive_key) {
            let bound = Value::Number(bound.clone());
            map.insert(bound_key.to_string(), bound);
            map.insert(exclusive_key.to_string(), Value::Bool(true));
        }
    }
}

/// Drop `null` path entries, `null` members inside path items
/// (`"delete": null`, `"parameters": null`), and `null` members inside
/// operations; generators emit them and `openapiv3` rejects them, which
/// used to fail the whole document.
fn strip_null_path_entries(value: &mut Value) {
    let Some(paths) = value.get_mut("paths").and_then(Value::as_object_mut) else {
        return;
    };
    paths.retain(|_, item| !item.is_null());
    for item in paths.values_mut() {
        let Some(item) = item.as_object_mut() else {
            continue;
        };
        item.retain(|_, member| !member.is_null());
        for operation in item.values_mut() {
            let Some(operation) = operation.as_object_mut() else {
                continue;
            };
            operation.retain(|_, member| !member.is_null());
            // A security requirement maps scheme name → scopes array; a
            // null scopes value means "no scopes" in the wild.
            if let Some(Value::Array(requirements)) = operation.get_mut("security") {
                for requirement in requirements {
                    if let Some(requirement) = requirement.as_object_mut() {
                        for scopes in requirement.values_mut() {
                            if scopes.is_null() {
                                *scopes = Value::Array(Vec::new());
                            }
                        }
                    }
                }
            }
        }
    }
}

fn normalize_object_type_union(map: &mut Map<String, Value>) {
    let Some(Value::Array(type_values)) = map.get(TYPE_KEY) else {
        return;
    };

    let mut non_null_types = Vec::new();
    let mut saw_null = false;
    for type_value in type_values {
        let Some(type_name) = type_value.as_str() else {
            return;
        };
        if type_name == NULL_TYPE_NAME {
            saw_null = true;
        } else {
            non_null_types.push(type_name.to_string());
        }
    }

    match non_null_types.as_slice() {
        [single_type] if saw_null => {
            map.insert(TYPE_KEY.to_string(), Value::String(single_type.clone()));
            map.insert(NULLABLE_KEY.to_string(), Value::Bool(true));
        }
        [single_type] => {
            map.insert(TYPE_KEY.to_string(), Value::String(single_type.clone()));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_real_world_sloppiness, normalized_openapiv3, parse_openapi_value, ref_repair,
    };
    use crate::{Generator, OpenApiDocument};
    use openapiv3::{ReferenceOr, SchemaKind, Type};
    use serde_json::json;

    #[test]
    fn sample_v30_frontends_generate_identical_output() {
        let sample_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../sample_openapi");
        let mut samples = schema_files(&sample_dir);
        samples.sort();

        for path in samples {
            assert_v30_frontends_match(&path);
        }
    }

    #[test]
    fn corpus_v30_frontends_generate_identical_output() {
        if std::env::var_os("PROGENITOR_FRONTEND_CONVERGENCE_CORPUS").is_none() {
            return;
        }

        let corpus_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.cache");
        let mut samples = schema_files(&corpus_dir);
        samples.sort();

        for path in samples {
            assert_v30_frontends_match(&path);
        }
    }

    fn schema_files(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
        std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                matches!(
                    path.extension().and_then(std::ffi::OsStr::to_str),
                    Some("json" | "yaml")
                )
            })
            .collect()
    }

    fn assert_v30_frontends_match(path: &std::path::Path) {
        let source = std::fs::read_to_string(path).unwrap();
        let mut value: serde_json::Value =
            if path.extension().and_then(std::ffi::OsStr::to_str) == Some("json") {
                serde_json::from_str(&source).unwrap()
            } else {
                serde_yaml::from_str(&source).unwrap()
            };
        if !value
            .get("openapi")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|version| version.trim().starts_with("3.0"))
        {
            return;
        }

        ref_repair::relocate_kind_mismatched_component_refs(&mut value);
        ref_repair::hoist_deep_pointer_refs(&mut value);
        normalize_real_world_sloppiness(&mut value);

        let legacy = normalized_openapiv3(value.clone()).unwrap();
        let legacy = OpenApiDocument(crate::ir::v30::lower(&legacy).unwrap());
        let unified = OpenApiDocument(crate::ir::v31::parse(value).unwrap());

        let legacy_output = Generator::default().generate_text(&legacy).unwrap();
        let unified_output = Generator::default().generate_text(&unified).unwrap();
        assert_eq!(
            legacy_output,
            unified_output,
            "frontend output differs for {}",
            path.display()
        );
    }

    #[test]
    fn parses_nullable_type_unions_from_openapi_3_1() {
        let openapi = normalized_openapiv3(json!({
            "openapi": "3.1.0",
            "info": {
                "title": "example",
                "version": "1.0.0"
            },
            "paths": {},
            "components": {
                "schemas": {
                    "Link": {
                        "type": "object",
                        "properties": {
                            "deprecation": {
                                "type": ["string", "null"]
                            }
                        }
                    }
                }
            }
        }))
        .expect("parse openapi");

        let components = openapi.components.expect("components");
        let ReferenceOr::Item(link_schema) = components.schemas["Link"].clone() else {
            panic!("expected inline Link schema");
        };
        let SchemaKind::Type(Type::Object(link_type)) = link_schema.schema_kind else {
            panic!("expected Link to be an object schema");
        };
        let ReferenceOr::Item(deprecation_schema) = link_type.properties["deprecation"].clone()
        else {
            panic!("expected inline deprecation schema");
        };

        assert!(deprecation_schema.schema_data.nullable);
        assert!(matches!(
            deprecation_schema.schema_kind,
            SchemaKind::Type(Type::String(_))
        ));
    }

    #[test]
    fn fills_missing_info_version() {
        let document = parse_openapi_value(json!({
            "openapi": "3.0.0",
            "info": {
                "title": "example"
            },
            "paths": {}
        }))
        .expect("parse openapi");

        assert_eq!(document.0.info.version, "unknown");
    }

    #[test]
    fn tolerates_null_response_description() {
        let document = parse_openapi_value(json!({
            "openapi": "3.0.0",
            "info": {
                "title": "example",
                "version": "1.0.0"
            },
            "paths": {
                "/things": {
                    "get": {
                        "operationId": "listThings",
                        "responses": {
                            "200": {
                                "description": null
                            }
                        }
                    }
                }
            }
        }))
        .expect("parse openapi");

        assert_eq!(document.0.operations[0].responses[0].description, "");
    }

    #[test]
    fn tolerates_non_query_deep_object_parameter_style() {
        let document = parse_openapi_value(json!({
            "openapi": "3.0.0",
            "info": {
                "title": "example",
                "version": "1.0.0"
            },
            "paths": {
                "/users/{user}": {
                    "get": {
                        "operationId": "getUser",
                        "parameters": [
                            {
                                "name": "user",
                                "in": "path",
                                "required": true,
                                "style": "deepObject",
                                "schema": {
                                    "type": "string"
                                }
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "ok"
                            }
                        }
                    }
                }
            }
        }))
        .expect("parse openapi");

        assert_eq!(document.0.operations[0].parameters[0].name, "user");
    }

    #[test]
    fn tolerates_empty_schema_type_string() {
        let document = parse_openapi_value(json!({
            "openapi": "3.1.0",
            "info": {
                "title": "example",
                "version": "1.0.0"
            },
            "paths": {},
            "components": {
                "schemas": {
                    "MaybeAnything": {
                        "type": ""
                    }
                }
            }
        }))
        .expect("parse openapi");

        assert!(document.0.schemas.contains_key("MaybeAnything"));
    }

    #[test]
    fn tolerates_unknown_schema_type_string() {
        let document = parse_openapi_value(json!({
            "openapi": "3.1.0",
            "info": {
                "title": "example",
                "version": "1.0.0"
            },
            "paths": {},
            "components": {
                "schemas": {
                    "MaybeAnything": {
                        "type": "unknown"
                    }
                }
            }
        }))
        .expect("parse openapi");

        assert!(document.0.schemas.contains_key("MaybeAnything"));
    }

    #[test]
    fn canonicalizes_schema_type_string_case() {
        let document = parse_openapi_value(json!({
            "openapi": "3.1.0",
            "info": {
                "title": "example",
                "version": "1.0.0"
            },
            "paths": {
                "/payment/checklist": {
                    "get": {
                        "operationId": "getPaymentChecklist",
                        "parameters": [
                            {
                                "name": "session",
                                "in": "cookie",
                                "schema": {
                                    "type": "String"
                                }
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "ok"
                            }
                        }
                    }
                }
            }
        }))
        .expect("parse openapi");

        assert_eq!(document.0.operations[0].parameters[0].name, "session");
    }
}
