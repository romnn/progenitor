use openapiv3::OpenAPI;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::ir::OpenApiDocument;

const OPENAPI_VERSION_KEY: &str = "openapi";
const NULL_TYPE_NAME: &str = "null";
const TYPE_KEY: &str = "type";
const NULLABLE_KEY: &str = "nullable";

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
            serde_yaml::from_str(document).map_err(|yaml_err| ParseOpenApiError::Format {
                json_message: json_err.to_string(),
                yaml_message: yaml_err.to_string(),
            })?
        }
    };

    parse_openapi_value(value)
}

/// Parse an already-decoded OpenAPI document of any supported spec version
/// into progenitor's internal model.
pub fn parse_openapi_value(
    value: Value,
) -> std::result::Result<OpenApiDocument, ParseOpenApiError> {
    // Both 3.0.x and 3.1.x currently go through the `openapiv3` frontend,
    // with 3.1 documents normalized into a 3.0-compatible shape first.
    // 3.1 will grow a native frontend that handles the full JSON Schema
    // 2020-12 surface.
    let spec = normalized_openapiv3(value)?;
    let document = crate::ir::v30::lower(&spec)?;
    Ok(OpenApiDocument(document))
}

/// Normalize a serde value into the subset of OpenAPI 3.1 that `openapiv3`
/// can represent directly, then deserialize it.
fn normalized_openapiv3(mut value: Value) -> std::result::Result<OpenAPI, ParseOpenApiError> {
    downcast_openapi_version(&mut value);
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

fn downcast_openapi_version(value: &mut Value) {
    let Some(map) = value.as_object_mut() else {
        return;
    };
    let needs_downgrade = map
        .get(OPENAPI_VERSION_KEY)
        .and_then(Value::as_str)
        .is_some_and(|version| version.starts_with("3.1"));
    if needs_downgrade {
        map.insert(
            OPENAPI_VERSION_KEY.to_string(),
            Value::String("3.0.3".to_string()),
        );
    }
}

fn normalize_nullable_type_unions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for entry in map.values_mut() {
                normalize_nullable_type_unions(entry);
            }
            normalize_object_type_union(map);
        }
        Value::Array(items) => {
            for item in items {
                normalize_nullable_type_unions(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
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
    use super::normalized_openapiv3;
    use openapiv3::{ReferenceOr, SchemaKind, Type};
    use serde_json::json;

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
}
