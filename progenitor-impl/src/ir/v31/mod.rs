// Copyright 2026 Oxide Computer Company

//! OpenAPI 3.1.x frontend: parses a 3.1 document with a tolerant serde
//! skeleton and lowers it into the version-agnostic [`super`] model.
//!
//! The document structure (paths, operations, parameters, bodies,
//! responses) is typed; schemas stay raw [`serde_json::Value`]s until the
//! dialect rewriter in [`schema`] lowers them from JSON Schema 2020-12 to
//! the draft-07 model typify consumes. Existing typed 3.1 parsers were
//! rejected because they silently drop the 2020-12 keywords wild specs
//! actually use (`$defs`, `patternProperties`, `contentMediaType`, …).

mod schema;

use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

use crate::openapi::ParseOpenApiError;
use crate::{Error, Result, ir};

use schema::SchemaLowering;

/// Dialect URIs whose keyword semantics match what the schema rewriter
/// assumes. Anything else would silently change keyword meaning.
const SUPPORTED_DIALECTS: [&str; 2] = [
    "https://json-schema.org/draft/2020-12/schema",
    "https://spec.openapis.org/oas/3.1/dialect/base",
];

pub(crate) fn parse(value: Value) -> std::result::Result<ir::Document, ParseOpenApiError> {
    let document: Document31 =
        serde_path_to_error::deserialize(value).map_err(|err| ParseOpenApiError::Deserialize {
            path: err.path().to_string(),
            message: err.inner().to_string(),
        })?;
    Ok(lower(document)?)
}

fn lower(document: Document31) -> Result<ir::Document> {
    validate_version(&document.openapi)?;
    if let Some(dialect) = &document.json_schema_dialect
        && !SUPPORTED_DIALECTS.contains(&dialect.as_str())
    {
        return Err(Error::UnexpectedFormat(format!(
            "unsupported jsonSchemaDialect: {dialect}"
        )));
    }

    let mut lowering = SchemaLowering::new(document.components.schemas.keys().cloned());
    let mut schemas = lowering.lower_components(document.components.schemas.clone())?;

    let mut operations = Vec::new();
    for (path, item_value) in &document.paths {
        // Generators emit `null` path entries; skip them like the 3.0
        // frontend does.
        if item_value.is_null() {
            continue;
        }
        let item = resolve_path_item(item_value, &document.components)?;
        let item: PathItem31 = from_value(item.clone(), &format!("path item {path}"))?;
        for (method, operation_value) in item.operations() {
            let operation: Operation31 =
                from_value(operation_value.clone(), &format!("{method} {path}"))?;
            operations.push(lower_operation(
                operation,
                &item.parameters,
                path,
                method,
                &document.components,
                &mut lowering,
                &mut schemas,
            )?);
        }
    }

    let tags = document
        .tags
        .into_iter()
        .map(|tag| ir::Tag {
            name: tag.name,
            description: tag.description,
        })
        .collect();

    let mut ir_document = ir::Document {
        info: ir::Info {
            title: document.info.title,
            version: document.info.version,
            description: document.info.description,
            terms_of_service: document.info.terms_of_service,
        },
        schemas,
        operations,
        tags,
    };
    ir::patch_dangling_schema_refs(&mut ir_document);
    ir::ensure_operation_ids(&mut ir_document);
    ir::validate(&ir_document)?;
    Ok(ir_document)
}

fn validate_version(version: &str) -> Result<()> {
    if version.trim().starts_with("3.1") {
        Ok(())
    } else {
        Err(Error::UnexpectedFormat(format!(
            "invalid version: {version}"
        )))
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_operation(
    operation: Operation31,
    path_parameters: &[Value],
    path: &str,
    method: &str,
    components: &Components31,
    lowering: &mut SchemaLowering,
    schemas: &mut IndexMap<String, schemars::schema::Schema>,
) -> Result<ir::Operation> {
    let context = format!("{method} {path}");

    // Merge path-item and operation parameters; operation parameters
    // override path-item parameters of the same name, and the result is
    // name-ordered — the same contract the 3.0 frontend honors.
    let mut merged: BTreeMap<String, Parameter31> = BTreeMap::new();
    for parameter_value in path_parameters.iter().chain(&operation.parameters) {
        let resolved = resolve_component(parameter_value, &components.parameters, "parameter")?;
        let parameter: Parameter31 =
            from_value(resolved.clone(), &format!("parameter in {context}"))?;
        merged.insert(parameter.name.clone(), parameter);
    }

    let parameters = merged
        .into_values()
        .map(|parameter| lower_parameter(parameter, &context, lowering, schemas))
        .collect::<Result<Vec<_>>>()?;

    let request_body = operation
        .request_body
        .as_ref()
        .map(|body_value| -> Result<ir::RequestBody> {
            let resolved =
                resolve_component(body_value, &components.request_bodies, "request body")?;
            let body: RequestBody31 =
                from_value(resolved.clone(), &format!("request body of {context}"))?;
            Ok(ir::RequestBody {
                description: body.description,
                required: body.required,
                content: lower_content(body.content, &context, lowering, schemas)?,
            })
        })
        .transpose()?;

    // The `default` response first, then explicit statuses in document
    // order — the ordering contract the generator relies on.
    let mut responses = Vec::new();
    let (default_entries, coded_entries): (Vec<_>, Vec<_>) = operation
        .responses
        .iter()
        .partition(|(status, _)| *status == "default");
    for (status_key, response_value) in default_entries.into_iter().chain(coded_entries) {
        let status = parse_status(status_key, &context)?;
        let resolved = resolve_component(response_value, &components.responses, "response")?;
        let response: Response31 = from_value(
            resolved.clone(),
            &format!("response {status_key} of {context}"),
        )?;
        responses.push(ir::Response {
            status,
            description: response.description,
            content: lower_content(response.content, &context, lowering, schemas)?,
        });
    }

    let extensions = operation
        .rest
        .into_iter()
        .filter(|(key, _)| key.starts_with("x-"))
        .collect();

    Ok(ir::Operation {
        operation_id: operation.operation_id,
        tags: operation.tags,
        method: method.to_string(),
        path: path.to_string(),
        summary: operation.summary,
        description: operation.description,
        parameters,
        request_body,
        responses,
        extensions,
    })
}

fn lower_parameter(
    parameter: Parameter31,
    context: &str,
    lowering: &mut SchemaLowering,
    schemas: &mut IndexMap<String, schemars::schema::Schema>,
) -> Result<ir::Parameter> {
    let style = parameter.style.as_deref();
    let kind = match parameter.location.as_str() {
        "path" => ir::ParameterKind::Path {
            style: match style {
                None | Some("simple") => ir::PathStyle::Simple,
                Some("matrix") => ir::PathStyle::Matrix,
                Some("label") => ir::PathStyle::Label,
                Some(other) => {
                    return Err(Error::UnexpectedFormat(format!(
                        "unknown path parameter style `{other}` in {context}"
                    )));
                }
            },
        },
        "query" => ir::ParameterKind::Query {
            style: match style {
                None | Some("form") => ir::QueryStyle::Form,
                Some("spaceDelimited") => ir::QueryStyle::SpaceDelimited,
                Some("pipeDelimited") => ir::QueryStyle::PipeDelimited,
                Some("deepObject") => ir::QueryStyle::DeepObject,
                Some(other) => {
                    return Err(Error::UnexpectedFormat(format!(
                        "unknown query parameter style `{other}` in {context}"
                    )));
                }
            },
            allow_reserved: parameter.allow_reserved,
            allow_empty_value: parameter.allow_empty_value,
        },
        "header" => ir::ParameterKind::Header {
            style: ir::HeaderStyle::Simple,
        },
        "cookie" => ir::ParameterKind::Cookie,
        other => {
            return Err(Error::UnexpectedFormat(format!(
                "unknown parameter location `{other}` in {context}"
            )));
        }
    };

    let schema = match (parameter.schema, parameter.content.is_some()) {
        // `content`-style parameters are unsupported; `None` makes the
        // generator report that, matching the 3.0 frontend.
        (_, true) => None,
        (Some(schema_value), false) => Some(lower_schema(
            schema_value,
            &format!("parameter {} in {context}", parameter.name),
            lowering,
            schemas,
        )?),
        (None, false) => None,
    };

    // The spec requires `required: true` on path parameters; wild specs
    // routinely omit it, and a path parameter cannot meaningfully be
    // optional, so normalize rather than panic downstream.
    let required = parameter.required || parameter.location == "path";

    Ok(ir::Parameter {
        name: parameter.name,
        description: parameter.description,
        required,
        kind,
        schema,
    })
}

fn lower_content(
    content: IndexMap<String, MediaType31>,
    context: &str,
    lowering: &mut SchemaLowering,
    schemas: &mut IndexMap<String, schemars::schema::Schema>,
) -> Result<IndexMap<String, ir::MediaTypeObject>> {
    content
        .into_iter()
        .map(|(media_type, mt)| {
            let schema = mt
                .schema
                .map(|schema_value| -> Result<ir::SchemaRef> {
                    // A direct `$ref` carries the component name through to
                    // the response-collapse logic, exactly like 3.0's
                    // `ReferenceOr::Reference`.
                    let ref_name = schema_value
                        .get("$ref")
                        .and_then(Value::as_str)
                        .and_then(|r| r.strip_prefix("#/components/schemas/"))
                        .filter(|name| !name.contains('/'))
                        .map(str::to_string);
                    let schema = lower_schema(
                        schema_value,
                        &format!("{media_type} schema in {context}"),
                        lowering,
                        schemas,
                    )?;
                    Ok(ir::SchemaRef { schema, ref_name })
                })
                .transpose()?;
            Ok((
                media_type,
                ir::MediaTypeObject {
                    schema,
                    has_encoding: !mt.encoding.is_empty(),
                },
            ))
        })
        .collect()
}

fn lower_schema(
    value: Value,
    context: &str,
    lowering: &mut SchemaLowering,
    schemas: &mut IndexMap<String, schemars::schema::Schema>,
) -> Result<schemars::schema::Schema> {
    let schema = lowering.lower_inline(value, context)?;
    for (name, hoisted) in lowering.take_pending_hoists() {
        schemas.insert(name, hoisted);
    }
    Ok(schema)
}

fn parse_status(status: &str, context: &str) -> Result<ir::ResponseStatus> {
    if status == "default" {
        return Ok(ir::ResponseStatus::Default);
    }
    if let Ok(code) = status.parse::<u16>() {
        return Ok(ir::ResponseStatus::Code(code));
    }
    let bytes = status.as_bytes();
    if bytes.len() == 3 && bytes[0].is_ascii_digit() && status[1..].eq_ignore_ascii_case("xx") {
        return Ok(ir::ResponseStatus::Range((bytes[0] - b'0') as u16));
    }
    Err(Error::UnexpectedFormat(format!(
        "invalid response status `{status}` in {context}"
    )))
}

/// Follow `$ref` chains through a components section by final path
/// segment, mirroring the 3.0 frontend's resolution semantics.
fn resolve_component<'a>(
    value: &'a Value,
    section: &'a IndexMap<String, Value>,
    what: &str,
) -> Result<&'a Value> {
    let mut current = value;
    let mut depth = 0u32;
    while let Some(reference) = current.get("$ref").and_then(Value::as_str) {
        let key = reference.rsplit('/').next().unwrap_or(reference);
        current = section.get(key).ok_or_else(|| {
            Error::UnexpectedFormat(format!("unresolved {what} reference: {reference}"))
        })?;
        depth += 1;
        if depth > 32 {
            return Err(Error::UnexpectedFormat(format!(
                "{what} reference cycle at: {reference}"
            )));
        }
    }
    Ok(current)
}

/// Path items have first-class `$ref` in 3.1 (resolving against
/// `components.pathItems`). An unresolved ref must be an error — with the
/// tolerant skeleton it would otherwise read as an empty path item and
/// its operations would silently vanish.
fn resolve_path_item<'a>(value: &'a Value, components: &'a Components31) -> Result<&'a Value> {
    resolve_component(value, &components.path_items, "path item")
}

fn from_value<T: serde::de::DeserializeOwned>(value: Value, context: &str) -> Result<T> {
    serde_path_to_error::deserialize(value).map_err(|err| {
        Error::UnexpectedFormat(format!(
            "invalid {context} at {}: {}",
            err.path(),
            err.inner()
        ))
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Document31 {
    #[serde(default)]
    openapi: String,
    info: Info31,
    #[serde(default)]
    json_schema_dialect: Option<String>,
    #[serde(default)]
    paths: IndexMap<String, Value>,
    #[serde(default)]
    components: Components31,
    #[serde(default)]
    tags: Vec<Tag31>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Info31 {
    #[serde(default)]
    title: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    terms_of_service: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Components31 {
    #[serde(default)]
    schemas: IndexMap<String, Value>,
    #[serde(default)]
    parameters: IndexMap<String, Value>,
    #[serde(default)]
    request_bodies: IndexMap<String, Value>,
    #[serde(default)]
    responses: IndexMap<String, Value>,
    #[serde(default)]
    path_items: IndexMap<String, Value>,
}

#[derive(Deserialize)]
struct Tag31 {
    name: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct PathItem31 {
    #[serde(default)]
    parameters: Vec<Value>,
    #[serde(default)]
    get: Option<Value>,
    #[serde(default)]
    put: Option<Value>,
    #[serde(default)]
    post: Option<Value>,
    #[serde(default)]
    delete: Option<Value>,
    #[serde(default)]
    options: Option<Value>,
    #[serde(default)]
    head: Option<Value>,
    #[serde(default)]
    patch: Option<Value>,
    #[serde(default)]
    trace: Option<Value>,
}

impl PathItem31 {
    /// Operations in the same fixed order the 3.0 frontend walks
    /// (`openapiv3::PathItem::iter`) — never JSON key order.
    fn operations(&self) -> impl Iterator<Item = (&'static str, &Value)> {
        [
            ("get", &self.get),
            ("put", &self.put),
            ("post", &self.post),
            ("delete", &self.delete),
            ("options", &self.options),
            ("head", &self.head),
            ("patch", &self.patch),
            ("trace", &self.trace),
        ]
        .into_iter()
        // `null` members ("delete": null) count as absent.
        .filter_map(|(method, op)| match op {
            Some(op) if !op.is_null() => Some((method, op)),
            _ => None,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Operation31 {
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    operation_id: Option<String>,
    #[serde(default)]
    parameters: Vec<Value>,
    #[serde(default)]
    request_body: Option<Value>,
    #[serde(default)]
    responses: IndexMap<String, Value>,
    #[serde(flatten)]
    rest: IndexMap<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Parameter31 {
    name: String,
    #[serde(rename = "in")]
    location: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    style: Option<String>,
    #[serde(default)]
    allow_reserved: bool,
    #[serde(default)]
    allow_empty_value: Option<bool>,
    #[serde(default)]
    schema: Option<Value>,
    #[serde(default)]
    content: Option<IndexMap<String, Value>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestBody31 {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    content: IndexMap<String, MediaType31>,
}

#[derive(Deserialize)]
struct Response31 {
    #[serde(default)]
    description: String,
    #[serde(default)]
    content: IndexMap<String, MediaType31>,
}

#[derive(Deserialize)]
struct MediaType31 {
    #[serde(default)]
    schema: Option<Value>,
    #[serde(default)]
    encoding: IndexMap<String, Value>,
}
