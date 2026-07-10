// Copyright 2025 Oxide Computer Company

use std::cmp::Ordering;

use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Error, Generator, PreparedIr, Result, TagStyle, ir,
    operation::{
        BodyContentType, DROPSHOT_LIMIT_PARAM, DROPSHOT_PAGE_TOKEN_PARAM, DropshotPagination,
        HttpMethod, OperationMethod, OperationParameter, OperationParameterKind,
        OperationParameterType, OperationResponse, OperationResponseKind, OperationResponseStatus,
        ResponseSide, is_json_content_type, synth_variant_name,
    },
    util::{Case, sanitize, unique_ident_from},
};

#[path = "emit/builder.rs"]
mod builder;
#[path = "emit/method.rs"]
mod emit;
#[path = "operations/lower.rs"]
mod lower;
#[path = "emit/positional.rs"]
mod positional;
#[path = "operations/responses.rs"]
mod responses;

use emit::{make_doc_comment, make_stream_doc_comment};
#[cfg(test)]
use lower::sort_params;
#[cfg(test)]
use responses::{collapse_bodyless_with_typed, find_common_supertype};
use responses::{success_arm_pattern, synth_decode_arm};

struct MethodSigBody {
    success: TokenStream,
    error: TokenStream,
    body: TokenStream,
    /// Definitions for any types synthesised by `method_sig_body` itself —
    /// today this is the per-operation `Status<code>` sum-type enums that
    /// `extract_responses` falls back to when the response set has multiple
    /// distinct kinds. Emitted by the caller alongside the function so the
    /// signature's `Synth("…")` identifier resolves.
    extra_types: TokenStream,
}

struct BuilderImpl {
    doc: String,
    sig: TokenStream,
    body: TokenStream,
}

struct BuilderParameter {
    name: proc_macro2::Ident,
    typ: TokenStream,
    initial_value: TokenStream,
    finalize: TokenStream,
    implementation: TokenStream,
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::collections::{BTreeMap, BTreeSet};
    use std::str::FromStr;

    use quote::quote;

    use crate::{Error, Generator, PreparedIr};

    use super::{
        BodyContentType, HttpMethod, OperationMethod, OperationParameter, OperationParameterKind,
        OperationParameterType, OperationResponse, OperationResponseKind, OperationResponseStatus,
        collapse_bodyless_with_typed, find_common_supertype, is_json_content_type, sort_params,
    };

    fn raw_parameter(api_name: &str, kind: OperationParameterKind) -> OperationParameter {
        OperationParameter {
            name: api_name.to_string(),
            api_name: api_name.to_string(),
            description: None,
            typ: OperationParameterType::RawBody,
            optional: false,
            inner_type_id: None,
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

    #[test]
    fn non_default_upgrade_error_returns_generation_error() {
        let method = OperationMethod {
            operation_id: "upgrade".to_string(),
            tags: Vec::new(),
            method: HttpMethod::Get,
            path: crate::template::parse("/").unwrap(),
            summary: None,
            description: None,
            params: Vec::new(),
            responses: vec![
                OperationResponse {
                    status_code: OperationResponseStatus::Code(200),
                    typ: OperationResponseKind::None,
                    schema_name: None,
                    media_type: None,
                    description: None,
                },
                OperationResponse {
                    status_code: OperationResponseStatus::Code(400),
                    typ: OperationResponseKind::Upgrade,
                    schema_name: None,
                    media_type: None,
                    description: None,
                },
            ],
            dropshot_paginated: None,
            dropshot_websocket: true,
        };

        let prepared = PreparedIr {
            raw_methods: Vec::new(),
            schema_supertypes: BTreeMap::new(),
            schema_type_ids: BTreeMap::new(),
            component_schemas: indexmap::IndexMap::new(),
        };
        let result = Generator::default().method_sig_body(
            &prepared,
            &method,
            quote! { Self },
            quote! { self },
            false,
        );
        let Err(Error::UnexpectedFormat(message)) = result else {
            panic!("expected unsupported upgrade response error");
        };
        assert!(message.contains("non-default error responses"));
    }

    fn kinds<const N: usize>(items: [OperationResponseKind; N]) -> BTreeSet<OperationResponseKind> {
        items.into_iter().collect()
    }

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
    fn collapse_bodyless_collapses_single_typed_plus_none() {
        let collapsed = collapse_bodyless_with_typed(&kinds([
            OperationResponseKind::Raw,
            OperationResponseKind::None,
        ]));
        assert_eq!(collapsed, Some(OperationResponseKind::Raw));

        let collapsed = collapse_bodyless_with_typed(&kinds([
            OperationResponseKind::Upgrade,
            OperationResponseKind::None,
        ]));
        assert_eq!(collapsed, Some(OperationResponseKind::Upgrade));
    }

    #[test]
    fn collapse_bodyless_leaves_uniform_sets_alone() {
        // Single kind — caller's existing path handles it.
        assert_eq!(
            collapse_bodyless_with_typed(&kinds([OperationResponseKind::Raw])),
            None
        );
        assert_eq!(
            collapse_bodyless_with_typed(&kinds([OperationResponseKind::None])),
            None
        );
        assert_eq!(collapse_bodyless_with_typed(&BTreeSet::new()), None);
    }

    #[test]
    fn collapse_bodyless_does_not_collapse_two_body_kinds() {
        // `Raw` + `Upgrade` is a genuine multi-kind conflict — the existing
        // assertion should still fire downstream so we surface the spec issue.
        assert_eq!(
            collapse_bodyless_with_typed(&kinds([
                OperationResponseKind::Raw,
                OperationResponseKind::Upgrade,
            ])),
            None,
        );
    }

    fn supertype_map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
        entries
            .iter()
            .map(|(child, parent)| ((*child).to_string(), (*parent).to_string()))
            .collect()
    }

    fn name_set(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn find_common_supertype_picks_immediate_parent_when_one_input_is_the_parent() {
        // BadRequestProblem extends Problem; both appear in the input set.
        // The LCA is Problem itself.
        let map = supertype_map(&[("BadRequestProblem", "Problem")]);
        assert_eq!(
            find_common_supertype(&name_set(&["BadRequestProblem", "Problem"]), &map),
            Some("Problem".to_string()),
        );
    }

    #[test]
    fn find_common_supertype_walks_multi_step_chains() {
        // A extends Middle extends Root; B extends Middle extends Root.
        // The LCA is Middle (deeper than Root).
        let map = supertype_map(&[("A", "Middle"), ("B", "Middle"), ("Middle", "Root")]);
        assert_eq!(
            find_common_supertype(&name_set(&["A", "B"]), &map),
            Some("Middle".to_string()),
        );
    }

    #[test]
    fn find_common_supertype_returns_none_when_no_shared_ancestor() {
        // X has no parent; Y has its own unrelated parent.
        let map = supertype_map(&[("Y", "OtherRoot")]);
        assert_eq!(find_common_supertype(&name_set(&["X", "Y"]), &map), None,);
    }

    #[test]
    fn find_common_supertype_tolerates_cycles_in_the_map() {
        // Pathological input: A → B → A. Walking must terminate.
        let map = supertype_map(&[("A", "B"), ("B", "A")]);
        // No shared ancestor with an unrelated type.
        assert_eq!(find_common_supertype(&name_set(&["A", "C"]), &map), None,);
    }

    #[test]
    fn find_common_supertype_handles_singleton_and_empty_sets() {
        let map = supertype_map(&[("A", "Root")]);
        // A single name's own chain trivially contains itself.
        assert_eq!(
            find_common_supertype(&name_set(&["A"]), &map),
            Some("A".to_string()),
        );
        assert_eq!(find_common_supertype(&BTreeSet::new(), &map), None);
    }

    #[test]
    fn collapse_bodyless_does_not_collapse_more_than_two_kinds() {
        // Even when `None` is present, three distinct kinds is not the
        // single-typed-plus-bodyless pattern the collapse is designed for.
        assert_eq!(
            collapse_bodyless_with_typed(&kinds([
                OperationResponseKind::Raw,
                OperationResponseKind::Upgrade,
                OperationResponseKind::None,
            ])),
            None,
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
