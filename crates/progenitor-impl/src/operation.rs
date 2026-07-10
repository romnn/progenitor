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
    pub typ: OperationParameterType,
    pub optional: bool,
    pub inner_type_id: Option<TypeId>,
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
/// and media types using the RFC 6839 `+json` structured syntax suffix.
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
    /// response references a named component schema.
    pub schema_name: Option<String>,
    /// The selected wire media type, when the response has content.
    pub media_type: Option<String>,
    // TODO this isn't currently used because dropshot doesn't give us a
    // particularly useful message here.
    #[allow(dead_code)]
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
            Self::Default | Self::Code(101) | Self::Code(200..=299) | Self::Range(2)
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
    /// Per-operation synthesized sum type used for response sets with multiple
    /// body-bearing kinds that cannot be collapsed.
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
