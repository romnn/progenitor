use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Generator, PreparedIr, ir,
    operation::{
        DROPSHOT_LIMIT_PARAM, DROPSHOT_PAGE_TOKEN_PARAM, DropshotPagination, OperationMethod,
        OperationParameter, OperationParameterKind, OperationParameterType, OperationResponse,
        OperationResponseKind, OperationResponseStatus, ResponseSide,
    },
    util::{Case, sanitize},
};

/// Find the lowest common ancestor of `names` in the inheritance graph
/// described by `supertype_map`. Returns the deepest ancestor present in
/// every input's supertype chain; `None` if no shared ancestor exists.
///
/// Walking is bounded: each chain is built only as far as the supertype
/// map carries the lookup, and cycles are detected via a contains-check.
pub(crate) fn find_common_supertype(
    names: &BTreeSet<String>,
    supertype_map: &BTreeMap<String, String>,
) -> Option<String> {
    if names.is_empty() {
        return None;
    }
    let chains: Vec<Vec<String>> = names
        .iter()
        .map(|name| {
            let mut chain = vec![name.clone()];
            let mut current = name.as_str();
            while let Some(parent) = supertype_map.get(current) {
                if chain.iter().any(|seen| seen == parent) {
                    break;
                }
                chain.push(parent.clone());
                current = parent.as_str();
            }
            chain
        })
        .collect();
    let first = chains.first()?;
    for candidate in first {
        if chains.iter().all(|chain| chain.contains(candidate)) {
            return Some(candidate.clone());
        }
    }
    None
}

/// If `types` is exactly `{kind, None}` where `kind != None`, return that
/// body-bearing kind. Otherwise return `None` (the caller should fall back
/// to its prior behaviour). See the call site in `extract_responses` for
/// the motivating context.
pub(super) fn collapse_bodyless_with_typed(
    types: &BTreeSet<OperationResponseKind>,
) -> Option<OperationResponseKind> {
    if types.len() != 2 || !types.contains(&OperationResponseKind::None) {
        return None;
    }
    types
        .iter()
        .find(|kind| !matches!(kind, OperationResponseKind::None))
        .cloned()
}

impl Generator {
    /// Extract responses for the requested side of an operation. The
    /// result is a `Vec<OperationResponse>` that enumerates the cases matching
    /// the filter, and an `OperationResponseKind` that represents the common
    /// generated type for those cases. Items may be rewritten relative to
    /// `method.responses` — see the bodyless-collapse, `allOf`-collapse, and
    /// multi-kind sum-type synthesis paths below.
    pub(crate) fn extract_responses(
        &self,
        prepared: &PreparedIr,
        method: &OperationMethod,
        side: ResponseSide,
    ) -> (Vec<OperationResponse>, OperationResponseKind) {
        let filter: fn(&OperationResponseStatus) -> bool = match side {
            ResponseSide::Success => OperationResponseStatus::is_success_or_default,
            ResponseSide::Error => OperationResponseStatus::is_error_or_default,
        };
        let mut response_items: Vec<OperationResponse> = method
            .responses
            .iter()
            .filter(|response| filter(&response.status_code))
            .cloned()
            .collect();
        response_items.sort();

        // If at least one 2xx response is declared and a `default` response
        // is also present, the `default` arm is unreachable from the success
        // path — every concrete success status is already handled by an
        // explicit code, and any 4xx/5xx that would otherwise fall through
        // to `default` is picked up by the error-side `extract_responses`
        // call anyway. Pop the trailing `default` so we don't trip the
        // unstable collapsed response signature with a
        // `{Type(success), Type(default)}` set. No-op for error-side calls
        // because the error filter excludes 2xx codes.
        let len = response_items.len();
        if len >= 2
            && matches!(
                response_items[len - 1].status_code,
                OperationResponseStatus::Default
            )
            && matches!(
                response_items[len - 2].status_code,
                OperationResponseStatus::Range(2) | OperationResponseStatus::Code(200..=299),
            )
        {
            response_items.pop();
        }

        // First pass: if every distinct `Type(...)` variant in the set
        // descends from a common ancestor schema via `allOf`, rewrite each
        // typed response to use that ancestor's `TypeId`. The runtime cost
        // is the loss of subtype-specific fields (e.g. `BadRequestProblem`'s
        // `violations` deserialized as `Problem`), which is acceptable
        // because the alternative is failing to generate the operation at
        // all. `None` / `Raw` / `Upgrade` items are passed through
        // untouched — they're handled by the subsequent bodyless collapse
        // or the strict assertion.
        let typed_schema_names: Option<BTreeSet<String>> = response_items
            .iter()
            .filter(|item| matches!(item.typ, OperationResponseKind::Type(_)))
            .map(|item| item.schema_name.clone())
            .collect();
        if let Some(names) = typed_schema_names
            && names.len() > 1
            && let Some(ancestor) = find_common_supertype(&names, &prepared.schema_supertypes)
            && let Some(type_id) = prepared.schema_type_ids.get(&ancestor)
        {
            for item in &mut response_items {
                if matches!(item.typ, OperationResponseKind::Type(_)) {
                    item.typ = OperationResponseKind::Type(type_id.clone());
                    item.schema_name = Some(ancestor.clone());
                }
            }
        }

        let response_types = response_items
            .iter()
            .map(|response| response.typ.clone())
            .collect::<BTreeSet<_>>();

        // Second pass: if the only distinct kinds are one body-bearing kind
        // plus `None` (a common pattern — e.g. 200 with a JSON body alongside
        // 304 / 204 / a bodyless Unauthorized), collapse to the body-bearing
        // kind and drop the `None` items from `response_items` so their
        // per-arm decodes are never emitted. Those status codes fall through
        // to the `_ => Err(Error::UnexpectedResponse(response))` catch-all
        // at runtime: callers still get an error tagged with the status
        // code, they just don't get a typed `ErrorResponse(ResponseValue<()>)`
        // variant for it.
        let response_type = if let Some(typed) = collapse_bodyless_with_typed(&response_types) {
            response_items.retain(|item| !matches!(item.typ, OperationResponseKind::None));
            typed
        } else if response_types.len() <= 1 {
            response_types
                .into_iter()
                .next()
                // TODO should this be OperationResponseType::Raw?
                .unwrap_or(OperationResponseKind::None)
        } else {
            // Genuine multi-kind set with no `allOf` ancestor available —
            // synthesise a per-operation enum whose variants are keyed by
            // status code. The enum definition itself is emitted by
            // `method_sig_body` based on the (still original) `typ`s in
            // `response_items`; here we only return the enum's name as the
            // common kind so the surrounding function signature compiles.
            let enum_name = format!(
                "{}{}",
                sanitize(&method.operation_id, Case::Pascal),
                match side {
                    ResponseSide::Success => "Response",
                    ResponseSide::Error => "Error",
                },
            );
            OperationResponseKind::Synth(enum_name)
        };
        (response_items, response_type)
    }

    // Validates all the necessary conditions for Dropshot pagination. Returns
    // the paginated item type data if all conditions are met.
    pub(super) fn dropshot_pagination_data(
        &self,
        operation: &ir::Operation,
        parameters: &[OperationParameter],
        responses: &[OperationResponse],
    ) -> Option<DropshotPagination> {
        let value = operation.extensions.get("x-dropshot-pagination")?;

        // We expect to see at least "page_token" and "limit" parameters.
        if parameters
            .iter()
            .filter(|param| {
                matches!(
                    (param.api_name.as_str(), &param.kind),
                    (
                        DROPSHOT_PAGE_TOKEN_PARAM | DROPSHOT_LIMIT_PARAM,
                        OperationParameterKind::Query { .. }
                    )
                ) && param.optional
            })
            .count()
            != 2
        {
            return None;
        }

        // All query parameters must be optional since page_token may not be
        // specified in conjunction with other query parameters.
        if !parameters.iter().all(|param| match &param.kind {
            OperationParameterKind::Query { .. } => param.optional,
            _ => true,
        }) {
            return None;
        }

        // A raw body parameter can only be passed to a single call as it may
        // be a streaming type. We can't use a streaming type for a paginated
        // interface because we can only stream it once rather than for the
        // multiple calls required to collect all pages.
        if parameters
            .iter()
            .any(|param| param.typ == OperationParameterType::RawBody)
        {
            return None;
        }

        // There must be exactly one successful response type.
        let mut success_response_items =
            responses
                .iter()
                .filter_map(|response| match (&response.status_code, &response.typ) {
                    (
                        OperationResponseStatus::Code(200..=299)
                        | OperationResponseStatus::Range(2),
                        OperationResponseKind::Type(type_id),
                    ) => Some(type_id),
                    _ => None,
                });

        let success_response = match (success_response_items.next(), success_response_items.next())
        {
            (None, _) | (_, Some(_)) => return None,
            (Some(success), None) => success,
        };

        let typ = self.type_space.get_type(success_response).ok()?;
        let details = match typ.details() {
            typify::TypeDetails::Struct(details) => details,
            _ => return None,
        };

        let properties = details.properties().collect::<BTreeMap<_, _>>();

        // There should be exactly two properties: items and next_page
        if properties.len() != 2 {
            return None;
        }

        // We need a next_page property that's an Option<String>.
        if let typify::TypeDetails::Option(ref opt_id) = self
            .type_space
            .get_type(properties.get("next_page")?)
            .ok()?
            .details()
        {
            if !matches!(
                self.type_space.get_type(opt_id).ok()?.details(),
                typify::TypeDetails::String
            ) {
                return None;
            }
        } else {
            return None;
        }

        match self
            .type_space
            .get_type(properties.get("items")?)
            .ok()?
            .details()
        {
            typify::TypeDetails::Vec(item) => {
                #[derive(serde::Deserialize, Default)]
                struct DropshotPaginationFormat {
                    required: Vec<String>,
                }
                let first_page_params =
                    serde_json::from_value::<DropshotPaginationFormat>(value.clone())
                        .unwrap_or_default()
                        .required;
                Some(DropshotPagination {
                    item,
                    first_page_params,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{collapse_bodyless_with_typed, find_common_supertype};
    use crate::operation::OperationResponseKind;

    fn kinds<const N: usize>(items: [OperationResponseKind; N]) -> BTreeSet<OperationResponseKind> {
        items.into_iter().collect()
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
}
