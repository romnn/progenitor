// Copyright 2026 Oxide Computer Company

//! OpenAPI 3.0.x frontend: lowers an [`openapiv3::OpenAPI`] document into
//! the version-agnostic [`super`] model.
//!
//! Schema conversion (including `nullable` handling and the
//! `x-discriminator` extension) is delegated to [`crate::to_schema`];
//! non-schema references are resolved here via [`crate::util`].

use openapiv3::{Components, OpenAPI, ReferenceOr};

use crate::to_schema::ToSchema;
use crate::util::{ReferenceOrExt, items, parameter_map};
use crate::{Error, Result, ir};

pub(crate) fn lower(spec: &OpenAPI) -> Result<ir::Document> {
    validate(spec)?;

    let schemas = spec
        .components
        .iter()
        .flat_map(|components| {
            components
                .schemas
                .iter()
                .map(|(name, ref_or_schema)| (name.clone(), ref_or_schema.to_schema()))
        })
        .collect();

    let operations = spec
        .paths
        .iter()
        .flat_map(|(path, ref_or_item)| {
            // validate() rejected externally defined path items.
            let item = ref_or_item.as_item().unwrap();
            item.iter().map(move |(method, operation)| {
                (path.as_str(), method, operation, &item.parameters)
            })
        })
        .map(|(path, method, operation, path_parameters)| {
            lower_operation(operation, &spec.components, path, method, path_parameters)
        })
        .collect::<Result<Vec<_>>>()?;

    let tags = spec
        .tags
        .iter()
        .map(|tag| ir::Tag {
            name: tag.name.clone(),
            description: tag.description.clone(),
        })
        .collect();

    let mut document = ir::Document {
        info: ir::Info {
            title: spec.info.title.clone(),
            version: spec.info.version.clone(),
            description: spec.info.description.clone(),
            terms_of_service: spec.info.terms_of_service.clone(),
        },
        schemas,
        operations,
        tags,
    };
    ir::patch_dangling_schema_refs(&mut document);
    ir::ensure_operation_ids(&mut document);
    ir::validate(&document)?;
    Ok(document)
}

/// Do some very basic checks of the OpenAPI document.
pub(crate) fn validate(spec: &OpenAPI) -> Result<()> {
    validate_version(spec.openapi.as_str())?;

    spec.paths.paths.iter().try_for_each(|p| match p.1 {
        ReferenceOr::Reference { reference: _ } => Err(Error::UnexpectedFormat(format!(
            "path {} uses reference, unsupported",
            p.0,
        ))),
        ReferenceOr::Item(_) => Ok(()),
    })
}

fn validate_version(spec_version: &str) -> Result<()> {
    if spec_version.trim().starts_with("3.0.") {
        Ok(())
    } else {
        Err(Error::UnexpectedFormat(format!(
            "invalid version: {}",
            spec_version
        )))
    }
}

fn lower_operation(
    operation: &openapiv3::Operation,
    components: &Option<Components>,
    path: &str,
    method: &str,
    path_parameters: &[ReferenceOr<openapiv3::Parameter>],
) -> Result<ir::Operation> {
    // Merge path-item and operation parameters; operation parameters
    // override path-item parameters with the same (name, location)
    // identity. The BTreeMap keying makes the result name-ordered, which
    // downstream code relies on.
    let mut merged = parameter_map(path_parameters, components)?;
    for operation_param in items(&operation.parameters, components) {
        let parameter = operation_param?;
        let location = match parameter {
            openapiv3::Parameter::Query { .. } => "query",
            openapiv3::Parameter::Header { .. } => "header",
            openapiv3::Parameter::Path { .. } => "path",
            openapiv3::Parameter::Cookie { .. } => "cookie",
        };
        merged.insert((&parameter.parameter_data_ref().name, location), parameter);
    }

    let parameters = merged
        .values()
        .map(|parameter| lower_parameter(parameter))
        .collect::<Vec<_>>();

    let request_body = operation
        .request_body
        .as_ref()
        .map(|body| -> Result<ir::RequestBody> {
            let body = body.item(components)?;
            Ok(ir::RequestBody {
                description: body.description.clone(),
                required: body.required,
                content: body
                    .content
                    .iter()
                    .map(|(media_type, mt)| (media_type.clone(), lower_media_type(mt)))
                    .collect(),
            })
        })
        .transpose()?;

    // The default response (if any) comes first, then explicit status
    // codes in document order — downstream response processing relies on
    // this ordering.
    let mut responses = Vec::new();
    if let Some(ref_or_response) = &operation.responses.default {
        responses.push(lower_response(
            ir::ResponseStatus::Default,
            ref_or_response.item(components)?,
        ));
    }
    for (status_code, ref_or_response) in &operation.responses.responses {
        let status = match status_code {
            openapiv3::StatusCode::Code(code) => ir::ResponseStatus::Code(*code),
            openapiv3::StatusCode::Range(range) => ir::ResponseStatus::Range(*range),
        };
        responses.push(lower_response(status, ref_or_response.item(components)?));
    }

    Ok(ir::Operation {
        operation_id: operation.operation_id.clone(),
        tags: operation.tags.clone(),
        method: method.to_string(),
        path: path.to_string(),
        summary: operation.summary.clone(),
        description: operation.description.clone(),
        parameters,
        request_body,
        responses,
        extensions: operation.extensions.clone(),
    })
}

fn lower_parameter(parameter: &openapiv3::Parameter) -> ir::Parameter {
    let (parameter_data, kind) = match parameter {
        openapiv3::Parameter::Path {
            parameter_data,
            style,
        } => (
            parameter_data,
            ir::ParameterKind::Path {
                style: match style {
                    openapiv3::PathStyle::Simple => ir::PathStyle::Simple,
                    openapiv3::PathStyle::Matrix => ir::PathStyle::Matrix,
                    openapiv3::PathStyle::Label => ir::PathStyle::Label,
                },
            },
        ),
        openapiv3::Parameter::Query {
            parameter_data,
            allow_reserved,
            style,
            allow_empty_value,
        } => (
            parameter_data,
            ir::ParameterKind::Query {
                style: match style {
                    openapiv3::QueryStyle::Form => ir::QueryStyle::Form,
                    openapiv3::QueryStyle::SpaceDelimited => ir::QueryStyle::SpaceDelimited,
                    openapiv3::QueryStyle::PipeDelimited => ir::QueryStyle::PipeDelimited,
                    openapiv3::QueryStyle::DeepObject => ir::QueryStyle::DeepObject,
                },
                allow_reserved: *allow_reserved,
                allow_empty_value: *allow_empty_value,
            },
        ),
        openapiv3::Parameter::Header {
            parameter_data,
            style: openapiv3::HeaderStyle::Simple,
        } => (
            parameter_data,
            ir::ParameterKind::Header {
                style: ir::HeaderStyle::Simple,
            },
        ),
        openapiv3::Parameter::Cookie { parameter_data, .. } => {
            (parameter_data, ir::ParameterKind::Cookie)
        }
    };

    let schema = match &parameter_data.format {
        openapiv3::ParameterSchemaOrContent::Schema(schema) => Some(schema.to_schema()),
        openapiv3::ParameterSchemaOrContent::Content(_) => None,
    };

    ir::Parameter {
        name: parameter_data.name.clone(),
        description: parameter_data.description.clone(),
        required: parameter_data.required,
        kind,
        schema,
    }
}

fn lower_media_type(media_type: &openapiv3::MediaType) -> ir::MediaTypeObject {
    ir::MediaTypeObject {
        schema: media_type.schema.as_ref().map(lower_schema_ref),
        has_encoding: !media_type.encoding.is_empty(),
    }
}

fn lower_schema_ref(schema: &ReferenceOr<openapiv3::Schema>) -> ir::SchemaRef {
    let ref_name = match schema {
        ReferenceOr::Reference { reference } => reference
            .strip_prefix("#/components/schemas/")
            .map(str::to_string),
        ReferenceOr::Item(_) => None,
    };
    ir::SchemaRef {
        schema: schema.to_schema(),
        ref_name,
    }
}

fn lower_response(status: ir::ResponseStatus, response: &openapiv3::Response) -> ir::Response {
    ir::Response {
        status,
        description: response.description.clone(),
        content: response
            .content
            .iter()
            .map(|(media_type, mt)| (media_type.clone(), lower_media_type(mt)))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::validate_version;

    #[test]
    fn test_validate_version() {
        assert!(validate_version("3.0.0").is_ok());
        assert!(validate_version("3.0.1").is_ok());
        assert!(validate_version("3.0.4").is_ok());
        assert!(validate_version("3.0.5-draft").is_ok());
        assert_eq!(
            validate_version("3.1.0").unwrap_err().to_string(),
            "unexpected or unhandled format in the OpenAPI document invalid version: 3.1.0"
        );
    }
}
