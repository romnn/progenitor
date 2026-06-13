// Copyright 2022 Oxide Computer Company

use std::collections::BTreeMap;

use indexmap::IndexMap;
use openapiv3::{Components, Parameter, ReferenceOr, RequestBody, Response, Schema};
use unicode_ident::{is_xid_continue, is_xid_start};

use crate::Result;

pub(crate) trait ReferenceOrExt<T: ComponentLookup> {
    fn item<'a>(&'a self, components: &'a Option<Components>) -> Result<&'a T>;
}
pub(crate) trait ComponentLookup: Sized {
    fn get_components(components: &Components) -> &IndexMap<String, ReferenceOr<Self>>;
}

impl<T: ComponentLookup> ReferenceOrExt<T> for openapiv3::ReferenceOr<T> {
    fn item<'a>(&'a self, components: &'a Option<Components>) -> Result<&'a T> {
        let mut current = self;
        // References may chain through components; bound the walk so a
        // reference cycle becomes an error rather than infinite recursion.
        for _ in 0..32 {
            match current {
                ReferenceOr::Item(item) => return Ok(item),
                ReferenceOr::Reference { reference } => {
                    let key = reference.rsplit('/').next().unwrap_or(reference);
                    let components = components.as_ref().ok_or_else(|| {
                        crate::Error::UnexpectedFormat(format!(
                            "reference {} but the document has no components",
                            reference,
                        ))
                    })?;
                    current = T::get_components(components).get(key).ok_or_else(|| {
                        crate::Error::UnexpectedFormat(format!(
                            "unresolved reference: {}",
                            reference,
                        ))
                    })?;
                }
            }
        }
        Err(crate::Error::UnexpectedFormat(
            "reference cycle in components".to_string(),
        ))
    }
}

pub(crate) fn items<'a, T>(
    refs: &'a [ReferenceOr<T>],
    components: &'a Option<Components>,
) -> impl Iterator<Item = Result<&'a T>>
where
    T: ComponentLookup,
{
    refs.iter().map(|r| r.item(components))
}

/// A parameter's identity is (name, location): wild specs (Elasticsearch)
/// declare the same name as both a path and a query parameter of one
/// operation, so keying by name alone would collapse them.
pub(crate) fn parameter_map<'a>(
    refs: &'a [ReferenceOr<Parameter>],
    components: &'a Option<Components>,
) -> Result<BTreeMap<(&'a String, &'static str), &'a Parameter>> {
    items(refs, components)
        .map(|res| {
            res.map(|param| {
                let location = match param {
                    Parameter::Query { .. } => "query",
                    Parameter::Header { .. } => "header",
                    Parameter::Path { .. } => "path",
                    Parameter::Cookie { .. } => "cookie",
                };
                ((&param.parameter_data_ref().name, location), param)
            })
        })
        .collect()
}

impl ComponentLookup for Parameter {
    fn get_components(components: &Components) -> &IndexMap<String, ReferenceOr<Self>> {
        &components.parameters
    }
}

impl ComponentLookup for RequestBody {
    fn get_components(components: &Components) -> &IndexMap<String, ReferenceOr<Self>> {
        &components.request_bodies
    }
}

impl ComponentLookup for Response {
    fn get_components(components: &Components) -> &IndexMap<String, ReferenceOr<Self>> {
        &components.responses
    }
}

impl ComponentLookup for Schema {
    fn get_components(components: &Components) -> &IndexMap<String, ReferenceOr<Self>> {
        &components.schemas
    }
}

pub(crate) enum Case {
    Pascal,
    Snake,
}

pub(crate) fn sanitize(input: &str, case: Case) -> String {
    use heck::{ToPascalCase, ToSnakeCase};
    let to_case = match case {
        Case::Pascal => str::to_pascal_case,
        Case::Snake => str::to_snake_case,
    };
    // If every case was special then none of them would be.
    let out = match input {
        "+1" => "plus1".to_string(),
        "-1" => "minus1".to_string(),
        _ => to_case(
            &input
                .replace('\'', "")
                .replace(|c: char| !is_xid_continue(c), "-"),
        ),
    };

    let out = match out.chars().next() {
        None => to_case("x"),
        Some(c) if is_xid_start(c) => out,
        Some(_) => format!("_{}", out),
    };

    // Make sure the string is a valid Rust identifier.
    if typify::accept_as_ident(&out) {
        out
    } else {
        format!("{}_", out)
    }
}

/// Given a desired name and a slice of proc_macro2::Ident, generate a new
/// Ident that is unique from the slice.
pub(crate) fn unique_ident_from(
    name: &str,
    identities: &[proc_macro2::Ident],
) -> proc_macro2::Ident {
    let mut name = name.to_string();

    loop {
        let ident = quote::format_ident!("{}", name);

        if !identities.contains(&ident) {
            return ident;
        }

        name.insert_str(0, "_");
    }
}

/// Rustdoc compiles untagged ``` fences in doc comments as Rust doctests.
/// Operation descriptions fence shell commands and payloads (GitHub's
/// code-scanning endpoints fence curl examples), which then fail
/// `cargo test` in every consumer crate. Tag bare fence openers as `text`
/// so the snippet stays documentation. Twin of the same neutralization in
/// typify's `metadata_description`, which covers schema-level docs.
///
/// Also normalises lines indented by 4+ spaces that are outside a ``` fence:
/// rustdoc treats such lines as indented code blocks and compiles them as
/// Rust doctests. Stripping excess indentation to at most 3 spaces prevents
/// that interpretation without losing the content.
pub(crate) fn neutralize_doc_fences(text: &str) -> String {
    let mut in_fence = false;
    let mut changed = false;
    let lines = text.split('\n').map(|line| {
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        if trimmed.starts_with("```") {
            // CommonMark only recognises a ``` fence opener/closer when it
            // has at most 3 spaces of leading indentation; at 4+ spaces it
            // becomes an indented code block whose content starts with ` `
            // (invalid Rust).  Strip the excess indentation on fence lines
            // too so Markdown sees them as fences, not code blocks.
            let prefix = if leading >= 4 {
                changed = true;
                "   "
            } else {
                &line[..leading]
            };
            if in_fence {
                in_fence = false;
                return format!("{prefix}{trimmed}");
            }
            in_fence = true;
            let info = trimmed.trim_start_matches('`');
            if info.trim().is_empty() {
                changed = true;
                return format!("{prefix}{}text", trimmed.trim_end_matches(|c: char| c.is_whitespace()));
            }
            return format!("{prefix}{trimmed}");
        }
        if in_fence || trimmed.is_empty() {
            return line.to_string();
        }
        // A leading tab also creates an indented code block in CommonMark
        // (tab = one tab-stop = 4 spaces; rustdoc compiles it as Rust).
        if line.starts_with('\t') {
            changed = true;
            return format!("   {}", line.trim_start_matches('\t'));
        }
        // Lines outside a fence with 4+ leading spaces are treated by
        // rustdoc as indented code blocks and compiled as Rust. Strip
        // excess indentation to at most 3 leading spaces.
        if leading >= 4 {
            changed = true;
            format!("   {trimmed}")
        } else {
            line.to_string()
        }
    });
    let result = lines.collect::<Vec<_>>().join("\n");
    if changed { result } else { text.to_string() }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_neutralize_doc_fences() {
        // A bare fence opener becomes ```text so rustdoc doesn't compile
        // the snippet as a Rust doctest; tagged fences and closers are
        // untouched.
        let text = "Upload it:\n\n```\ngzip -c analysis-data.sarif | base64 -w0\n```\n\nor:\n\n```shell\ncurl -X POST\n```";
        let expected = "Upload it:\n\n```text\ngzip -c analysis-data.sarif | base64 -w0\n```\n\nor:\n\n```shell\ncurl -X POST\n```";
        assert_eq!(super::neutralize_doc_fences(text), expected);

        let plain = "Just words with `inline code`.";
        assert_eq!(super::neutralize_doc_fences(plain), plain);

        // Lines with 4+ leading spaces outside a fence are indented code
        // blocks in rustdoc and get compiled as Rust. Strip them to ≤ 3 spaces.
        let indented = "- `page_size`:\n        Request this many records per page\n\n        This value is automatically clamped.\n- `next`: next page";
        let neutralized = super::neutralize_doc_fences(indented);
        assert!(
            !neutralized.contains("        "),
            "8-space indent should have been stripped: {neutralized:?}"
        );
        assert!(
            neutralized.contains("   Request this many records per page"),
            "content should be preserved with ≤3 spaces"
        );
    }
}
