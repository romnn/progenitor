// Copyright 2026 Oxide Computer Company

//! Resolved operations shared by every code-generation backend.

use std::{cmp::Ordering, str::FromStr};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use typify::{TypeId, TypeSpace};

use crate::{Error, Result, template::PathTemplate};

/// The intermediate representation of an operation that will become a method.
pub(crate) struct OperationMethod {
    pub operation_id: String,
    pub tags: Vec<String>,
    pub method: HttpMethod,
    pub path: PathTemplate,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub params: Vec<OperationParameter>,
    pub responses: Vec<OperationResponse>,
    pub dropshot_paginated: Option<DropshotPagination>,
    pub(crate) dropshot_websocket: bool,
}

pub(crate) enum HttpMethod {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

impl FromStr for HttpMethod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "get" => Ok(Self::Get),
            "put" => Ok(Self::Put),
            "post" => Ok(Self::Post),
            "delete" => Ok(Self::Delete),
            "options" => Ok(Self::Options),
            "head" => Ok(Self::Head),
            "patch" => Ok(Self::Patch),
            "trace" => Ok(Self::Trace),
            _ => Err(Error::InternalError(format!("bad method: {s}"))),
        }
    }
}

impl HttpMethod {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Put => "put",
            Self::Post => "post",
            Self::Delete => "delete",
            Self::Options => "options",
            Self::Head => "head",
            Self::Patch => "patch",
            Self::Trace => "trace",
        }
    }

    pub(crate) fn routing_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.as_str())
    }

    pub(crate) fn httpmock_tokens(&self) -> TokenStream {
        match self {
            Self::Get => quote! { ::httpmock::Method::GET },
            Self::Put => quote! { ::httpmock::Method::PUT },
            Self::Post => quote! { ::httpmock::Method::POST },
            Self::Delete => quote! { ::httpmock::Method::DELETE },
            Self::Options => quote! { ::httpmock::Method::OPTIONS },
            Self::Head => quote! { ::httpmock::Method::HEAD },
            Self::Patch => quote! { ::httpmock::Method::PATCH },
            Self::Trace => quote! { ::httpmock::Method::TRACE },
        }
    }
}

pub(crate) struct DropshotPagination {
    pub item: TypeId,
    pub first_page_params: Vec<String>,
}

pub(crate) const DROPSHOT_PAGE_TOKEN_PARAM: &str = "page_token";
pub(crate) const DROPSHOT_LIMIT_PARAM: &str = "limit";

pub(crate) struct OperationParameter {
    /// Sanitized parameter name.
    pub name: String,
    /// Original parameter name provided by the API.
    pub api_name: String,
    pub description: Option<String>,
    /// The parameter's Rust type. For path/query/header/cookie parameters,
    /// lowering has already unwrapped an `Option<T>` schema type to `T`
    /// (optionality lives in `optional`) and degraded non-`Display` types
    /// to a plain string. Body parameters keep their type verbatim —
    /// including any `Option` wrapper — and are always `optional: false`;
    /// backends that need the unwrapped body type inspect
    /// `TypeDetails::Option` themselves.
    pub typ: OperationParameterType,
    /// Whether the caller may omit this parameter, folding together the
    /// spec's `required` flag and a nullable (`Option`-typed) schema.
    pub optional: bool,
    pub kind: OperationParameterKind,
}

#[derive(Eq, PartialEq)]
pub(crate) enum OperationParameterType {
    Type(TypeId),
    RawBody,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum OperationParameterKind {
    Path,
    Query { required: bool, deep_object: bool },
    Header { required: bool },
    Cookie { required: bool },
    // TODO bodies may be optional
    Body(BodyContentType),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BodyContentType {
    OctetStream,
    Json,
    FormUrlencoded,
    Text(String),
    /// Any other media type (multipart/form-data, application/yaml,
    /// image/*, …). The generated method takes a raw `reqwest::Body` and
    /// sets this content type verbatim — the caller is responsible for
    /// producing a conforming payload. Coarse, but it keeps one exotic
    /// upload endpoint from failing generation of the whole client.
    Raw(String),
}

/// Returns true for the canonical JSON media type, its parameterized forms
/// (`application/json;charset=utf-8`, `application/json;version=1.0`, ...),
/// and any media type using the RFC 6839 §3.1 `+json` structured syntax
/// suffix (`application/problem+json`, `application/vnd.foo.v1+json`,
/// `application/scim+json`, ...).
pub(crate) fn is_json_content_type(content_type: &str) -> bool {
    let base = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim();
    base == "application/json" || base.ends_with("+json")
}

impl FromStr for BodyContentType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let offset = s.find(';').unwrap_or(s.len());
        let base = &s[..offset];
        if is_json_content_type(s) {
            return Ok(Self::Json);
        }
        match base {
            "application/octet-stream" => Ok(Self::OctetStream),
            "application/x-www-form-urlencoded" => Ok(Self::FormUrlencoded),
            "text/plain" | "text/x-markdown" => Ok(Self::Text(String::from(base))),
            _ => Ok(Self::Raw(String::from(base))),
        }
    }
}

impl std::fmt::Display for BodyContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::OctetStream => "application/octet-stream",
            Self::Json => "application/json",
            Self::FormUrlencoded => "application/x-www-form-urlencoded",
            Self::Text(typ) | Self::Raw(typ) => typ,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OperationResponse {
    pub status_code: OperationResponseStatus,
    pub typ: OperationResponseKind,
    /// Source `components.schemas.<name>` of this response body, when the
    /// response references a named component schema. `None` for inline
    /// schemas and for bodyless / raw / upgrade responses. Used by
    /// `extract_responses` to detect sibling response types that share a
    /// common `allOf` ancestor and can be collapsed to that ancestor.
    pub schema_name: Option<String>,
    /// The wire media type that produced this response's `typ`, when one was
    /// selected from the response content map. `None` for bodyless / upgrade
    /// responses and the synthesized success fallback. Captured from the same
    /// content entry that set `typ` (not a re-scan) so kind and media type can't
    /// disagree; used by server generation to set the `content-type` of a raw
    /// passthrough response.
    pub media_type: Option<String>,
    // TODO this isn't currently used because dropshot doesn't give us a
    // particularly useful message here.
    #[expect(
        dead_code,
        reason = "response descriptions are retained until dropshot can surface them"
    )]
    pub(crate) description: Option<String>,
}

impl Eq for OperationResponse {}

impl PartialEq for OperationResponse {
    fn eq(&self, other: &Self) -> bool {
        self.status_code == other.status_code
    }
}

impl Ord for OperationResponse {
    fn cmp(&self, other: &Self) -> Ordering {
        self.status_code.cmp(&other.status_code)
    }
}

impl PartialOrd for OperationResponse {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum OperationResponseStatus {
    Code(u16),
    Range(u16),
    Default,
}

impl OperationResponseStatus {
    // Keep the ordering total and aligned with generated match-arm precedence:
    // exact statuses in a bucket first, then that bucket's range, then default.
    fn sort_key(&self) -> (u16, u8) {
        match self {
            Self::Code(code) => {
                assert!(*code < 1000);
                (*code, 0)
            }
            Self::Range(range) => {
                assert!(*range < 10);
                (*range * 100 + 99, 1)
            }
            Self::Default => (1000, 2),
        }
    }

    pub(crate) fn is_success_or_default(&self) -> bool {
        matches!(
            self,
            Self::Default | Self::Code(101 | 200..=299) | Self::Range(2)
        )
    }

    pub(crate) fn is_error_or_default(&self) -> bool {
        matches!(
            self,
            Self::Default | Self::Code(400..=599) | Self::Range(4..=5)
        )
    }

    pub(crate) fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl Ord for OperationResponseStatus {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

impl PartialOrd for OperationResponseStatus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub(crate) enum OperationResponseKind {
    Type(TypeId),
    None,
    Raw,
    Upgrade,
    /// Per-operation synthesized, status-keyed response type.
    ///
    /// Client generation uses this when payload kinds cannot collapse to one
    /// type. Server generation also uses it whenever a single exact status
    /// does not imply the response status. The string is the synthesized
    /// enum's Rust identifier.
    Synth(String),
}

impl OperationResponseKind {
    pub(crate) fn into_tokens(self, type_space: &TypeSpace) -> TokenStream {
        match self {
            Self::Type(ref type_id) => {
                let type_name = type_space.get_type(type_id).unwrap().ident();
                quote! { #type_name }
            }
            Self::None => quote! { () },
            Self::Raw => quote! { ByteStream },
            Self::Upgrade => quote! { reqwest::Upgraded },
            Self::Synth(name) => {
                let ident = format_ident!("{}", name);
                quote! { #ident }
            }
        }
    }
}

/// Translate a response status into a synthesized enum variant identifier.
pub(crate) fn synth_variant_name(status: &OperationResponseStatus) -> String {
    match status {
        OperationResponseStatus::Code(code) => format!("Status{code}"),
        OperationResponseStatus::Range(range) => format!("StatusRange{range}xx"),
        OperationResponseStatus::Default => "Default".to_string(),
    }
}

/// Which side of an operation's response set to resolve.
#[derive(Copy, Clone, Debug)]
pub(crate) enum ResponseSide {
    Success,
    Error,
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::str::FromStr;

    use super::{BodyContentType, OperationResponseStatus, is_json_content_type};

    #[test]
    fn response_status_order_is_total_and_specific_before_range() {
        let mut statuses = vec![
            OperationResponseStatus::Default,
            OperationResponseStatus::Range(4),
            OperationResponseStatus::Code(500),
            OperationResponseStatus::Code(401),
            OperationResponseStatus::Code(400),
            OperationResponseStatus::Range(5),
        ];
        statuses.sort();

        assert_eq!(
            statuses,
            vec![
                OperationResponseStatus::Code(400),
                OperationResponseStatus::Code(401),
                OperationResponseStatus::Range(4),
                OperationResponseStatus::Code(500),
                OperationResponseStatus::Range(5),
                OperationResponseStatus::Default,
            ],
        );
        assert_ne!(
            OperationResponseStatus::Code(400).cmp(&OperationResponseStatus::Range(4)),
            Ordering::Equal,
        );
    }

    #[test]
    fn json_content_type_matches_canonical_and_parameterized() {
        assert!(is_json_content_type("application/json"));
        assert!(is_json_content_type("application/json;charset=utf-8"));
        assert!(is_json_content_type("application/json; charset=utf-8"));
        assert!(is_json_content_type("application/json;version=1.0"));
    }

    #[test]
    fn json_content_type_matches_rfc6839_structured_syntax_suffix() {
        // RFC 6839 §3.1 — the `+json` structured syntax suffix.
        assert!(is_json_content_type("application/problem+json"));
        assert!(is_json_content_type("application/vnd.github.v3.star+json"));
        assert!(is_json_content_type("application/scim+json"));
        assert!(is_json_content_type("application/ld+json"));
        // Parameters after the suffix still parse correctly.
        assert!(is_json_content_type(
            "application/problem+json; charset=utf-8"
        ));
    }

    #[test]
    fn json_content_type_rejects_non_json() {
        assert!(!is_json_content_type("application/octet-stream"));
        assert!(!is_json_content_type("application/xml"));
        assert!(!is_json_content_type("text/plain"));
        assert!(!is_json_content_type("application/x-www-form-urlencoded"));
        // `+jsonish` is not a structured syntax suffix.
        assert!(!is_json_content_type("application/foo+jsonish"));
    }

    #[test]
    fn body_content_type_parses_rfc6839_suffix_as_json() {
        assert!(matches!(
            BodyContentType::from_str("application/problem+json").unwrap(),
            BodyContentType::Json,
        ));
        assert!(matches!(
            BodyContentType::from_str("application/vnd.api+json; charset=utf-8").unwrap(),
            BodyContentType::Json,
        ));
    }
}
