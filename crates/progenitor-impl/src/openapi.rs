use serde_json::Value;
use thiserror::Error;

use crate::ir::OpenApiDocument;

mod ref_repair;

const TYPE_KEY: &str = "type";

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
    /// Deserializing into the tolerant document skeleton failed at a specific path.
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

    let document = crate::ir::frontend::parse(value)?;
    Ok(OpenApiDocument(document))
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

#[cfg(test)]
mod tests {
    use super::parse_openapi_value;
    use serde_json::json;

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
