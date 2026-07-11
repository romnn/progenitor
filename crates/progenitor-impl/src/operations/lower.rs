use std::{cmp::Ordering, collections::BTreeMap, str::FromStr};

use indexmap::IndexMap;

use crate::{
    Error, Generator, Result, ir,
    operation::{
        BodyContentType, HttpMethod, OperationMethod, OperationParameter, OperationParameterKind,
        OperationParameterType, OperationResponse, OperationResponseKind, OperationResponseStatus,
        is_json_content_type,
    },
    util::{Case, sanitize},
};

impl Generator {
    pub(crate) fn process_operation(
        &mut self,
        operation: &ir::Operation,
        schemas: &IndexMap<String, schemars::schema::Schema>,
    ) -> Result<OperationMethod> {
        let operation_id = operation.operation_id.as_ref().unwrap();

        // Parameters arrive pre-merged (path-item + operation, operation
        // wins) and name-ordered from the frontend lowering.
        let mut params = operation
            .parameters
            .iter()
            .map(|parameter| {
                match &parameter.kind {
                    ir::ParameterKind::Path {
                        style: ir::PathStyle::Simple,
                    } => {
                        let schema = parameter_schema(parameter);

                        let name = sanitize(
                            &format!("{}-{}", operation_id, &parameter.name),
                            Case::Pascal,
                        );
                        let type_id = self.type_space.add_type_with_name(&schema, Some(name))?;

                        // A path parameter can't meaningfully be absent; if
                        // a (sloppy) nullable schema produced an Option,
                        // use the inner type so the generated path encoding
                        // operates on a concrete value.
                        let ty = self.type_space.get_type(&type_id).unwrap();
                        let type_id = if let typify::TypeDetails::Option(inner) = ty.details() {
                            inner
                        } else {
                            type_id
                        };

                        // The generated path encoding renders the value
                        // with `Display`. Wild specs declare path
                        // parameters whose types can't (Keycloak ships a
                        // free-form `{type: object}`, Mollie a nullable
                        // enum); degrade those to a plain string the
                        // caller formats rather than emitting code that
                        // doesn't compile.
                        let ty = self.type_space.get_type(&type_id).unwrap();
                        let type_id = if ty.has_impl(typify::TypeSpaceImpl::Display) {
                            type_id
                        } else {
                            let string: schemars::schema::Schema = schemars::schema::SchemaObject {
                                instance_type: Some(schemars::schema::InstanceType::String.into()),
                                ..Default::default()
                            }
                            .into();
                            self.type_space.add_type(&string)?
                        };

                        Ok(OperationParameter {
                            name: sanitize(&parameter.name, Case::Snake),
                            api_name: parameter.name.clone(),
                            description: parameter.description.clone(),
                            typ: OperationParameterType::Type(type_id),
                            optional: false,
                            kind: OperationParameterKind::Path,
                        })
                    }
                    ir::ParameterKind::Query {
                        style: style @ (ir::QueryStyle::Form | ir::QueryStyle::DeepObject),
                        // We always encode reserved chars; allow_empty_value
                        // is irrelevant for this client.
                        ..
                    } => {
                        let deep_object_query = *style == ir::QueryStyle::DeepObject;
                        let schema = parameter_schema(parameter);
                        let name = sanitize(
                            &format!("{}-{}", operation_id, &parameter.name),
                            Case::Pascal,
                        );

                        let type_id = self.type_space.add_type_with_name(&schema, Some(name))?;

                        let ty = self.type_space.get_type(&type_id).unwrap();

                        // If the type is itself optional, then we'll treat it
                        // as optional (irrespective of the `required` field on
                        // the parameter) and use the "inner" type.
                        let details = ty.details();
                        let (type_id, required) =
                            if let typify::TypeDetails::Option(inner) = details {
                                (inner, false)
                            } else {
                                (type_id, parameter.required)
                            };

                        Ok(OperationParameter {
                            name: sanitize(&parameter.name, Case::Snake),
                            api_name: parameter.name.clone(),
                            description: parameter.description.clone(),
                            typ: OperationParameterType::Type(type_id),
                            optional: !required,
                            kind: OperationParameterKind::Query {
                                required,
                                deep_object: deep_object_query,
                            },
                        })
                    }
                    ir::ParameterKind::Header {
                        style: ir::HeaderStyle::Simple,
                    } => {
                        let schema = parameter_schema(parameter);
                        let name = sanitize(
                            &format!("{}-{}", operation_id, &parameter.name),
                            Case::Pascal,
                        );

                        let type_id = self.type_space.add_type_with_name(&schema, Some(name))?;

                        // Same Option handling as query parameters: a
                        // nullable schema means the header is optional, and
                        // the generated header encoding needs the inner
                        // type (calling `.to_string()` on an `Option` does
                        // not compile).
                        let (type_id, required) = {
                            let ty = self.type_space.get_type(&type_id).unwrap();
                            if let typify::TypeDetails::Option(inner) = ty.details() {
                                (inner, false)
                            } else {
                                (type_id, parameter.required)
                            }
                        };

                        // The header encoding also renders with `Display`.
                        // Linode types its X-Filter header as a free-form
                        // JSON object; degrade such parameters to a plain
                        // string the caller serializes, exactly like path
                        // parameters.
                        let has_display = {
                            let ty = self.type_space.get_type(&type_id).unwrap();
                            ty.has_impl(typify::TypeSpaceImpl::Display)
                        };
                        let type_id = if has_display {
                            type_id
                        } else {
                            let string: schemars::schema::Schema = schemars::schema::SchemaObject {
                                instance_type: Some(schemars::schema::InstanceType::String.into()),
                                ..Default::default()
                            }
                            .into();
                            self.type_space.add_type(&string)?
                        };

                        Ok(OperationParameter {
                            name: sanitize(&parameter.name, Case::Snake),
                            api_name: parameter.name.clone(),
                            description: parameter.description.clone(),
                            typ: OperationParameterType::Type(type_id),
                            optional: !required,
                            kind: OperationParameterKind::Header { required },
                        })
                    }
                    ir::ParameterKind::Cookie => {
                        let schema = parameter_schema(parameter);
                        let name = sanitize(
                            &format!("{}-{}", operation_id, &parameter.name),
                            Case::Pascal,
                        );

                        let type_id = self.type_space.add_type_with_name(&schema, Some(name))?;

                        let (type_id, required) = {
                            let ty = self.type_space.get_type(&type_id).unwrap();
                            if let typify::TypeDetails::Option(inner) = ty.details() {
                                (inner, false)
                            } else {
                                (type_id, parameter.required)
                            }
                        };

                        let has_display = {
                            let ty = self.type_space.get_type(&type_id).unwrap();
                            ty.has_impl(typify::TypeSpaceImpl::Display)
                        };
                        let type_id = if has_display {
                            type_id
                        } else {
                            let string: schemars::schema::Schema = schemars::schema::SchemaObject {
                                instance_type: Some(schemars::schema::InstanceType::String.into()),
                                ..Default::default()
                            }
                            .into();
                            self.type_space.add_type(&string)?
                        };

                        Ok(OperationParameter {
                            name: sanitize(&parameter.name, Case::Snake),
                            api_name: parameter.name.clone(),
                            description: parameter.description.clone(),
                            typ: OperationParameterType::Type(type_id),
                            optional: !required,
                            kind: OperationParameterKind::Cookie { required },
                        })
                    }
                    ir::ParameterKind::Path { style } => Err(Error::UnexpectedFormat(format!(
                        "unsupported style of path parameter {style:#?}",
                    ))),
                    ir::ParameterKind::Query { style, .. } => Err(Error::UnexpectedFormat(
                        format!("unsupported style of query parameter {style:#?}"),
                    )),
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let dropshot_websocket = operation.extensions.get("x-dropshot-websocket").is_some();
        if dropshot_websocket {
            self.uses_websockets = true;
        }

        if let Some(body_param) = self.get_body_param(operation, schemas)? {
            params.push(body_param);
        }

        let tmp = crate::template::parse(&operation.path)?;
        let names = tmp.names();

        // Wild specs declare path parameters that don't appear in the URL
        // template; such a value has nowhere to go, so drop the parameter
        // rather than panic.
        params.retain(|param| {
            !matches!(param.kind, OperationParameterKind::Path) || names.contains(&param.api_name)
        });

        // The reverse also happens: the template names a parameter nobody
        // declared. Synthesize a string parameter so the URL can still be
        // constructed.
        for name in &names {
            let declared = params.iter().any(|param| {
                matches!(param.kind, OperationParameterKind::Path) && &param.api_name == name
            });
            if !declared {
                let schema: schemars::schema::Schema = schemars::schema::SchemaObject {
                    instance_type: Some(schemars::schema::InstanceType::String.into()),
                    ..Default::default()
                }
                .into();
                let type_name = sanitize(&format!("{operation_id}-{name}"), Case::Pascal);
                let type_id = self
                    .type_space
                    .add_type_with_name(&schema, Some(type_name))?;
                params.push(OperationParameter {
                    name: sanitize(name, Case::Snake),
                    api_name: name.clone(),
                    description: None,
                    typ: OperationParameterType::Type(type_id),
                    optional: false,
                    kind: OperationParameterKind::Path,
                });
            }
        }

        // Distinct API parameter names can sanitize to the same Rust
        // identifier (`filter.workflowId` and `filter.workflow_id` both
        // become `filter_workflow_id`, and Elasticsearch declares
        // `scroll_id` as both a path and a query parameter); disambiguate
        // deterministically so the generated function signature compiles.
        // This must run after every source of parameters — declared, body,
        // and template-synthesized — has been collected.
        let mut seen_names = std::collections::HashSet::new();
        for param in &mut params {
            let base = param.name.clone();
            let mut counter = 2;
            while !seen_names.insert(param.name.clone()) {
                param.name = format!("{base}_{counter}");
                counter += 1;
            }
        }

        sort_params(&mut params, &names)?;

        let mut success = false;

        let mut responses = operation
            .responses
            .iter()
            .map(|response| {
                let mut status_code = match response.status {
                    ir::ResponseStatus::Default => OperationResponseStatus::Default,
                    ir::ResponseStatus::Code(code) => OperationResponseStatus::Code(code),
                    ir::ResponseStatus::Range(range) => OperationResponseStatus::Range(range),
                };

                // A previous version of dropshot websockets failed to
                // properly report responses; clean this up here.
                if dropshot_websocket && status_code == OperationResponseStatus::Default {
                    status_code = OperationResponseStatus::Code(101);
                }

                // We categorize responses as "typed" based on a JSON
                // content type (canonical `application/json`, a
                // parameterized form like `application/json;version=1.0`,
                // or any RFC 6839 `+json` suffix like
                // `application/problem+json`), "upgrade" if it's a
                // websocket channel without a meaningful content-type,
                // "raw" if there's any other response content type (we
                // don't investigate further), or "none" if there is no
                // content.
                // TODO if there are multiple response content types we
                // could treat those like different response types and
                // create an enum; the generated client method would
                // check for the content type of the response just as it
                // currently examines the status code.
                let mut schema_name: Option<String> = None;
                // Captured from the same content selection that sets `typ`, so
                // the recorded media type can't disagree with the kind.
                let mut media_type: Option<String> = None;
                let json_entry = response
                    .content
                    .iter()
                    .find(|(content_type, _)| is_json_content_type(content_type));
                let typ = if let Some((content_type, mt)) = json_entry {
                    media_type = Some(content_type.clone());
                    // An `encoding` map on a JSON response has no defined
                    // meaning; ignore it rather than reject specs that
                    // carry one anyway.
                    if let Some(schema_ref) = &mt.schema {
                        // Capture the source component name when the
                        // response body is a `$ref` so `extract_responses`
                        // can later collapse sibling responses that share
                        // a common `allOf` ancestor.
                        schema_name = schema_ref.ref_name.clone();
                        let name = sanitize(
                            &format!("{}-response", operation.operation_id.as_ref().unwrap()),
                            Case::Pascal,
                        );
                        let type_id = self
                            .type_space
                            .add_type_with_name(&schema_ref.schema, Some(name))?;
                        OperationResponseKind::Type(type_id)
                    } else {
                        // A JSON media type with no `schema` is common
                        // in real-world specs ("returns some JSON, but
                        // we won't promise the shape"). Treat it as a
                        // raw byte stream — callers who want JSON can
                        // deserialize manually. Replacing the prior
                        // `todo!()` lets the operation generate at all.
                        OperationResponseKind::Raw
                    }
                } else if status_code == OperationResponseStatus::Code(101) {
                    OperationResponseKind::Upgrade
                } else if !response.content.is_empty() {
                    // Non-JSON content: pick a deterministic media type (the
                    // lexicographically smallest key) so server raw passthrough
                    // sets a stable `content-type` regardless of document order.
                    media_type = response.content.keys().min().cloned();
                    OperationResponseKind::Raw
                } else {
                    OperationResponseKind::None
                };

                // See if there's a status code that covers success cases.
                if matches!(
                    status_code,
                    OperationResponseStatus::Default
                        | OperationResponseStatus::Code(101 | 200..=299)
                        | OperationResponseStatus::Range(2)
                ) {
                    success = true;
                }

                let description = if response.description.is_empty() {
                    None
                } else {
                    Some(response.description.clone())
                };

                Ok(OperationResponse {
                    status_code,
                    typ,
                    schema_name,
                    media_type,
                    description,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        // If the API has declined to specify the characteristics of a
        // successful response, we cons up a generic one. Note that this is
        // technically permissible within OpenAPI, but advised against by the
        // spec.
        if !success {
            responses.push(OperationResponse {
                status_code: OperationResponseStatus::Range(2),
                typ: OperationResponseKind::Raw,
                schema_name: None,
                media_type: None,
                description: None,
            });
        }

        let dropshot_paginated = self.dropshot_pagination_data(operation, &params, &responses);

        if dropshot_websocket && dropshot_paginated.is_some() {
            return Err(Error::InvalidExtension(format!(
                "conflicting extensions in {operation_id:?}"
            )));
        }
        if dropshot_websocket
            && responses
                .iter()
                .find(|r| r.status_code == OperationResponseStatus::Code(101))
                .is_none()
        {
            return Err(Error::InvalidExtension(format!(
                "websocket endpoint {operation_id:?} must include an explicit 101 response code"
            )));
        }

        Ok(OperationMethod {
            operation_id: sanitize(operation_id, Case::Snake),
            tags: operation.tags.clone(),
            method: HttpMethod::from_str(&operation.method)?,
            path: tmp,
            summary: operation.summary.clone().filter(|s| !s.is_empty()),
            description: operation.description.clone().filter(|s| !s.is_empty()),
            params,
            responses,
            dropshot_paginated,
            dropshot_websocket,
        })
    }

    fn get_body_param(
        &mut self,
        operation: &ir::Operation,
        schemas: &IndexMap<String, schemars::schema::Schema>,
    ) -> Result<Option<OperationParameter>> {
        let body = match &operation.request_body {
            Some(body) => body,
            None => return Ok(None),
        };

        let (content_str, media_type) = match (body.content.first(), body.content.len()) {
            (None, _) => return Ok(None),
            (Some(first), 1) => first,
            // Multiple request-body media types is common in real-world specs
            // (e.g. an endpoint advertising both `application/json` for a
            // typed body and `multipart/form-data` for a binary upload).
            // Progenitor can only generate one body parameter per operation
            // today (multipart support is incomplete — see oxidecomputer/
            // progenitor#418), so prefer the canonical JSON variant when
            // present; otherwise fall back to the first declared variant.
            // The other variants are not exposed in the generated client.
            (_, _) => body
                .content
                .iter()
                .find(|(name, _)| is_json_content_type(name))
                .or_else(|| body.content.first())
                .expect("non-empty content map was checked above"),
        };

        let mut content_type = BodyContentType::from_str(content_str)?;

        // A JSON or form body without a schema can't be typed; fall back to
        // the raw passthrough treatment rather than failing the operation
        // ("returns/accepts some JSON, no promised shape" is common in the
        // wild — the response side has the same fallback).
        if matches!(
            content_type,
            BodyContentType::Json | BodyContentType::FormUrlencoded
        ) && media_type.schema.is_none()
        {
            content_type = BodyContentType::Raw(
                content_str
                    .split(';')
                    .next()
                    .unwrap_or(content_str)
                    .to_string(),
            );
        }

        let typ = match content_type {
            BodyContentType::OctetStream => {
                // A raw binary body is either schema-less (the canonical
                // OpenAPI 3.1 form) or a plain string schema marked binary:
                // 3.0's `format: binary`, or 3.1's `contentEncoding` /
                // `contentMediaType` keywords. Wild specs also attach
                // fancier shapes (Cloudflare's Workers KV PUT body is
                // `anyOf: [string, string+binary]`); the media type already
                // dictates that the wire payload is raw bytes, so any
                // schema here is advisory — never grounds to fail the
                // operation.
                OperationParameterType::RawBody
            }
            BodyContentType::Text(ref text_type) => {
                // For a plain text body, we expect no schema or a simple
                // string. Anything fancier (an object schema, a
                // contentEncoding like base64) means the payload isn't the
                // plain string we'd send; degrade to the raw passthrough
                // treatment so the caller controls the exact bytes.
                let plain = match &media_type.schema {
                    None => true,
                    Some(schema_ref) => {
                        let resolved = ir::resolve_schema(&schema_ref.schema, schemas);
                        is_plain_string_schema(resolved, None) && !has_content_keywords(resolved)
                    }
                };
                if !plain {
                    content_type = BodyContentType::Raw(text_type.clone());
                }
                OperationParameterType::RawBody
            }
            // The payload of any other media type is opaque to the
            // generator; expose it as a raw body regardless of what the
            // schema says.
            BodyContentType::Raw(_) => OperationParameterType::RawBody,
            BodyContentType::Json | BodyContentType::FormUrlencoded => {
                let schema_ref = media_type.schema.as_ref().ok_or_else(|| {
                    Error::UnexpectedFormat("No schema specified for request body".to_string())
                })?;
                // The `encoding` map describes per-property serialization
                // (style/explode/contentType) for form payloads. The
                // default serde_urlencoded serialization matches the
                // common cases; honoring the long tail isn't worth failing
                // generation of the whole client, so encodings are
                // intentionally ignored here.
                let name = sanitize(
                    &format!("{}-body", operation.operation_id.as_ref().unwrap()),
                    Case::Pascal,
                );
                let typ = self
                    .type_space
                    .add_type_with_name(&schema_ref.schema, Some(name))?;
                OperationParameterType::Type(typ)
            }
        };

        Ok(Some(OperationParameter {
            name: "body".to_string(),
            api_name: "body".to_string(),
            description: body.description.clone(),
            typ,
            optional: false,
            kind: OperationParameterKind::Body(content_type),
        }))
    }
}

/// Get a parameter's schema.
///
/// Parameters that use `content` rather than `schema` serialize their
/// value according to a media type (usually JSON). Model them as plain
/// strings the caller fills with the already-serialized form — coarse,
/// but far better than failing the whole client over one exotic
/// parameter.
fn parameter_schema(parameter: &ir::Parameter) -> std::borrow::Cow<'_, schemars::schema::Schema> {
    match &parameter.schema {
        Some(schema) => std::borrow::Cow::Borrowed(schema),
        None => std::borrow::Cow::Owned(
            schemars::schema::SchemaObject {
                instance_type: Some(schemars::schema::InstanceType::String.into()),
                ..Default::default()
            }
            .into(),
        ),
    }
}

/// A binary payload schema: `type: string` marked binary via 3.0's
/// `format: binary` or 3.1's `contentEncoding`/`contentMediaType` keywords
/// (which land in the schema's extensions).
fn has_content_keywords(schema: &schemars::schema::Schema) -> bool {
    let schemars::schema::Schema::Object(object) = schema else {
        return false;
    };
    object.extensions.contains_key("contentEncoding")
        || object.extensions.contains_key("contentMediaType")
}

/// Check that a (resolved) schema is exactly the plain string shape the
/// raw-body paths require: `type: string` with the given `format` and no
/// other constraints. Descriptive metadata (title, description, examples)
/// is fine; anything that would affect the value space is not. This
/// mirrors the openapiv3 shape the generator historically pattern-matched
/// before schemas moved to their schemars representation.
fn is_plain_string_schema(schema: &schemars::schema::Schema, format: Option<&str>) -> bool {
    let schemars::schema::Schema::Object(object) = schema else {
        return false;
    };
    let type_is_string = matches!(
        &object.instance_type,
        Some(schemars::schema::SingleOrVec::Single(single))
            if **single == schemars::schema::InstanceType::String
    );
    let format_matches = object.format.as_deref() == format;
    let no_string_constraints = object.string.as_ref().is_none_or(|string| {
        string.pattern.is_none() && string.min_length.is_none() && string.max_length.is_none()
    });
    let no_other_kind = object.subschemas.is_none()
        && object.object.is_none()
        && object.array.is_none()
        && object.number.is_none()
        && object.enum_values.is_none()
        && object.const_value.is_none()
        && object.reference.is_none();
    let no_value_constraints = object
        .metadata
        .as_ref()
        .is_none_or(|metadata| metadata.default.is_none())
        && !object
            .extensions
            .contains_key(typify::DISCRIMINATOR_EXTENSION_KEY);
    type_is_string
        && format_matches
        && no_string_constraints
        && no_other_kind
        && no_value_constraints
}

pub(super) fn sort_params(raw_params: &mut [OperationParameter], names: &[String]) -> Result<()> {
    for param in raw_params
        .iter()
        .filter(|param| matches!(param.kind, OperationParameterKind::Path))
    {
        if !names.contains(&param.api_name) {
            return Err(Error::InvalidPath(format!(
                "parameter {} is missing from the path template",
                param.api_name
            )));
        }
    }
    for name in names {
        if !raw_params.iter().any(|param| {
            matches!(param.kind, OperationParameterKind::Path) && param.api_name == *name
        }) {
            return Err(Error::InvalidPath(format!(
                "path template parameter {name} is not declared"
            )));
        }
    }
    if raw_params
        .iter()
        .filter(|param| matches!(param.kind, OperationParameterKind::Body(_)))
        .count()
        > 1
    {
        return Err(Error::UnexpectedFormat(
            "operation declares more than one request body".to_string(),
        ));
    }

    let path_positions = names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    raw_params.sort_by(
        |OperationParameter {
             kind: a_kind,
             api_name: a_name,
             ..
         },
         OperationParameter {
             kind: b_kind,
             api_name: b_name,
             ..
         }| {
            match (a_kind, b_kind) {
                // Path params are first and are in positional order.
                (OperationParameterKind::Path, OperationParameterKind::Path) => path_positions
                    .get(a_name.as_str())
                    .cmp(&path_positions.get(b_name.as_str())),
                (OperationParameterKind::Path, OperationParameterKind::Query { .. }) => {
                    Ordering::Less
                }
                (OperationParameterKind::Path, OperationParameterKind::Body(_)) => Ordering::Less,
                (OperationParameterKind::Path, OperationParameterKind::Header { .. }) => {
                    Ordering::Less
                }
                (OperationParameterKind::Path, OperationParameterKind::Cookie { .. }) => {
                    Ordering::Less
                }

                // Query params are in lexicographic order.
                (OperationParameterKind::Query { .. }, OperationParameterKind::Body(_)) => {
                    Ordering::Less
                }
                (OperationParameterKind::Query { .. }, OperationParameterKind::Query { .. }) => {
                    a_name.cmp(b_name)
                }
                (OperationParameterKind::Query { .. }, OperationParameterKind::Path) => {
                    Ordering::Greater
                }
                (OperationParameterKind::Query { .. }, OperationParameterKind::Header { .. }) => {
                    Ordering::Less
                }
                (OperationParameterKind::Query { .. }, OperationParameterKind::Cookie { .. }) => {
                    Ordering::Less
                }

                // Body params are last and should be singular.
                (OperationParameterKind::Body(_), OperationParameterKind::Path) => {
                    Ordering::Greater
                }
                (OperationParameterKind::Body(_), OperationParameterKind::Query { .. }) => {
                    Ordering::Greater
                }
                (OperationParameterKind::Body(_), OperationParameterKind::Header { .. }) => {
                    Ordering::Greater
                }
                (OperationParameterKind::Body(_), OperationParameterKind::Cookie { .. }) => {
                    Ordering::Greater
                }
                (OperationParameterKind::Body(_), OperationParameterKind::Body(_)) => {
                    Ordering::Equal
                }

                // Header and cookie params are in lexicographic order.
                (OperationParameterKind::Header { .. }, OperationParameterKind::Header { .. }) => {
                    a_name.cmp(b_name)
                }
                (OperationParameterKind::Header { .. }, OperationParameterKind::Cookie { .. }) => {
                    a_name.cmp(b_name)
                }
                (OperationParameterKind::Header { .. }, _) => Ordering::Greater,
                (OperationParameterKind::Cookie { .. }, OperationParameterKind::Cookie { .. }) => {
                    a_name.cmp(b_name)
                }
                (OperationParameterKind::Cookie { .. }, OperationParameterKind::Header { .. }) => {
                    a_name.cmp(b_name)
                }
                (OperationParameterKind::Cookie { .. }, _) => Ordering::Greater,
            }
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::sort_params;
    use crate::Error;
    use crate::operation::{
        BodyContentType, OperationParameter, OperationParameterKind, OperationParameterType,
    };

    fn raw_parameter(api_name: &str, kind: OperationParameterKind) -> OperationParameter {
        OperationParameter {
            name: api_name.to_string(),
            api_name: api_name.to_string(),
            description: None,
            typ: OperationParameterType::RawBody,
            optional: false,
            kind,
        }
    }

    #[test]
    fn sort_params_rejects_path_parameter_missing_from_template() {
        let mut params = [raw_parameter("missing", OperationParameterKind::Path)];

        let result = sort_params(&mut params, &[]);

        assert!(matches!(result, Err(Error::InvalidPath(_))));
    }

    #[test]
    fn sort_params_rejects_undeclared_template_parameter() {
        let mut params = [];

        let result = sort_params(&mut params, &["id".to_string()]);

        assert!(matches!(result, Err(Error::InvalidPath(_))));
    }

    #[test]
    fn sort_params_rejects_duplicate_bodies() {
        let mut params = [
            raw_parameter("first", OperationParameterKind::Body(BodyContentType::Json)),
            raw_parameter(
                "second",
                OperationParameterKind::Body(BodyContentType::Json),
            ),
        ];

        let result = sort_params(&mut params, &[]);

        assert!(matches!(result, Err(Error::UnexpectedFormat(_))));
    }
}
