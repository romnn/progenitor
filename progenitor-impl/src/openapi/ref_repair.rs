//! Raw-value repairs for malformed `$ref`s, applied to the decoded
//! `serde_json::Value` before version dispatch so both the 3.0 and 3.1
//! frontends benefit.
//!
//! Two classes of wild-spec damage are handled here:
//!
//! - components filed under the wrong `components` kind (PagerDuty stores
//!   a response-shaped object under `components.requestBodies` and
//!   references it from response positions);
//! - JSON-pointer `$ref`s that reach *inside* a component body, such as
//!   `#/components/schemas/Account/definitions/accountPrototype`. Both
//!   resolvers only support whole-component references, so the targeted
//!   subtrees are hoisted into `components.schemas` and every ref to them
//!   rewritten. Hoisting (rather than inlining) keeps self-referential
//!   targets finite and guarantees one shared definition per pointer no
//!   matter how many sites reference it.

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};

const REF_KEY: &str = "$ref";
const COMPONENT_REF_PREFIX: &str = "#/components/";

/// Authors sometimes file a component under the wrong `components` kind
/// and reference it from a position that demands the other kind. Both
/// resolvers look refs up strictly by kind, which fails the whole
/// document even though the intent is unambiguous. Copy each misfiled
/// component to the kind its reference position implies and retarget the
/// mismatched refs; a real component of the expected kind is never
/// clobbered.
pub(super) fn relocate_kind_mismatched_component_refs(doc: &mut Value) {
    relocate_misfiled_components(doc, RefPosition::Response);
    relocate_misfiled_components(doc, RefPosition::RequestBody);
}

/// The position a `$ref` appears in, which dictates the component kind it
/// must resolve against.
#[derive(Clone, Copy)]
enum RefPosition {
    /// `paths/*/*/responses/*` or `components.responses/*`.
    Response,
    /// `paths/*/*/requestBody` or `components.requestBodies/*`.
    RequestBody,
}

impl RefPosition {
    /// The component kind this position requires.
    fn expected_kind(self) -> &'static str {
        match self {
            Self::Response => "responses",
            Self::RequestBody => "requestBodies",
        }
    }

    /// The opposite kind, where a mismatched ref actually points.
    fn misfiled_kind(self) -> &'static str {
        match self {
            Self::Response => "requestBodies",
            Self::RequestBody => "responses",
        }
    }
}

fn relocate_misfiled_components(doc: &mut Value, position: RefPosition) {
    let wrong_prefix = format!("{COMPONENT_REF_PREFIX}{}/", position.misfiled_kind());

    // Raw (still RFC6901-encoded) name segments of misfiled refs, in
    // document order. Only whole-component refs qualify; deeper pointers
    // are the hoisting pass's job.
    let mut misfiled: Vec<String> = Vec::new();
    for_each_positioned_ref(doc, position, &mut |reference| {
        if let Some(tail) = reference.strip_prefix(&wrong_prefix)
            && !tail.is_empty()
            && !tail.contains('/')
            && !misfiled.iter().any(|seen| seen == tail)
        {
            misfiled.push(tail.to_string());
        }
    });

    let mut renames: HashMap<String, String> = HashMap::new();
    for tail in misfiled {
        let name = decode_pointer_segment(&tail);
        if copy_misfiled_component(doc, position, &name) {
            renames.insert(
                format!("{wrong_prefix}{tail}"),
                format!("{COMPONENT_REF_PREFIX}{}/{tail}", position.expected_kind()),
            );
        }
    }
    if renames.is_empty() {
        return;
    }

    // Rewrite only refs in the mismatched position: the same ref string in
    // a position matching its kind is correct and must keep pointing at
    // the original component.
    for_each_positioned_ref(doc, position, &mut |reference| {
        if let Some(new_ref) = renames.get(reference.as_str()) {
            *reference = new_ref.clone();
        }
    });
}

fn for_each_positioned_ref(
    doc: &mut Value,
    position: RefPosition,
    visit: &mut dyn FnMut(&mut String),
) {
    if let Some(paths) = doc.get_mut("paths").and_then(Value::as_object_mut) {
        for path_item in paths.values_mut() {
            let Some(path_item) = path_item.as_object_mut() else {
                continue;
            };
            // Only operations are objects with the members probed below;
            // other path-item members (parameters, servers, summary, …)
            // fall through the `as_object_mut`/`get_mut` filters.
            for operation in path_item.values_mut() {
                let Some(operation) = operation.as_object_mut() else {
                    continue;
                };
                match position {
                    RefPosition::Response => {
                        let responses = operation
                            .get_mut("responses")
                            .and_then(Value::as_object_mut);
                        for response in responses.into_iter().flat_map(Map::values_mut) {
                            visit_ref_string(response, visit);
                        }
                    }
                    RefPosition::RequestBody => {
                        if let Some(body) = operation.get_mut("requestBody") {
                            visit_ref_string(body, visit);
                        }
                    }
                }
            }
        }
    }

    let components_of_kind = doc
        .get_mut("components")
        .and_then(|components| components.get_mut(position.expected_kind()))
        .and_then(Value::as_object_mut);
    for entry in components_of_kind.into_iter().flat_map(Map::values_mut) {
        visit_ref_string(entry, visit);
    }
}

fn visit_ref_string(value: &mut Value, visit: &mut dyn FnMut(&mut String)) {
    if let Some(Value::String(reference)) = value
        .as_object_mut()
        .and_then(|object| object.get_mut(REF_KEY))
    {
        visit(reference);
    }
}

fn copy_misfiled_component(doc: &mut Value, position: RefPosition, name: &str) -> bool {
    let Some(components) = doc.get_mut("components").and_then(Value::as_object_mut) else {
        return false;
    };
    // Only repair when the expected kind genuinely lacks the component; a
    // same-named component of the right kind wins over the misfiled one.
    if components
        .get(position.expected_kind())
        .and_then(|kind| kind.get(name))
        .is_some()
    {
        return false;
    }
    let Some(original) = components
        .get(position.misfiled_kind())
        .and_then(|kind| kind.get(name))
    else {
        return false;
    };

    let mut copy = original.clone();
    if let Some(object) = copy.as_object_mut() {
        match position {
            RefPosition::Response => {
                // openapiv3::Response requires a description.
                if !object.contains_key("description") {
                    object.insert("description".to_string(), Value::String(String::new()));
                }
            }
            RefPosition::RequestBody => {
                // Response-only fields a request body cannot carry.
                object.remove("headers");
                object.remove("links");
            }
        }
    }

    if matches!(position, RefPosition::RequestBody) {
        // The misfiled original stays behind under `components.responses`,
        // where openapiv3::Response still demands a description even if
        // nothing references the entry anymore. A request-body-shaped
        // object naturally lacks one, which would fail the whole 3.0
        // parse, so patch the original in place.
        if let Some(original) = components
            .get_mut(position.misfiled_kind())
            .and_then(|kind| kind.get_mut(name))
            .and_then(Value::as_object_mut)
            && !original.contains_key("description")
        {
            original.insert("description".to_string(), Value::String(String::new()));
        }
    }

    let target = components
        .entry(position.expected_kind())
        .or_insert_with(|| Value::Object(Map::new()));
    let Some(target) = target.as_object_mut() else {
        return false;
    };
    target.insert(name.to_string(), copy);
    true
}

/// Hoist every `$ref` that points *inside* a component body
/// (`#/components/<kind>/<name>/<deeper…>`) into a synthetic entry under
/// `components.schemas`, replacing the original subtree with a ref to the
/// new entry and rewriting all sites that used the deep pointer.
///
/// Refs that sit inside example literals are data, not references, and are
/// neither collected nor rewritten. Pointers that do not resolve (or
/// resolve to something that cannot stand as a schema) are left untouched
/// for the dangling-reference machinery to report.
pub(super) fn hoist_deep_pointer_refs(doc: &mut Value) {
    let mut pointers: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    walk_refs(doc, WalkMode::Keywords, &mut |reference: &mut String| {
        if is_deep_component_pointer(reference) && seen.insert(reference.clone()) {
            pointers.push(reference.clone());
        }
    });
    if pointers.is_empty() {
        return;
    }

    // Synthetic names are assigned in document order so generated output
    // is deterministic (serde_json's preserve_order is on workspace-wide).
    let assignments = assign_synthetic_names(doc, &pointers);
    if assignments.is_empty() {
        return;
    }

    // Extract deepest-first: a pointer nested inside another pointer's
    // target must be hoisted before the parent subtree moves, otherwise
    // the parent extraction would tear the child's target out of the
    // document. A nested pointer always has more segments than its parent.
    let mut extraction_order: Vec<usize> = (0..assignments.len()).collect();
    extraction_order.sort_by_key(|&index| std::cmp::Reverse(segment_count(&assignments[index].0)));

    let mut bodies: Vec<Option<Value>> = vec![None; assignments.len()];
    for index in extraction_order {
        let (pointer, name) = &assignments[index];
        // serde_json::pointer_mut performs RFC6901 evaluation, including
        // ~1//~0 decoding of segments like `application~1json`.
        let Some(target) = doc.pointer_mut(&pointer[1..]) else {
            continue;
        };
        let stub = Value::Object(Map::from_iter([(
            REF_KEY.to_string(),
            Value::String(format!("{COMPONENT_REF_PREFIX}schemas/{name}")),
        )]));
        bodies[index] = Some(std::mem::replace(target, stub));
    }

    let mut renames: HashMap<String, String> = HashMap::new();
    if let Some(schemas) = schemas_map(doc) {
        for (index, (pointer, name)) in assignments.iter().enumerate() {
            if let Some(body) = bodies[index].take() {
                schemas.insert(name.clone(), body);
                renames.insert(
                    pointer.clone(),
                    format!("{COMPONENT_REF_PREFIX}schemas/{name}"),
                );
            }
        }
    }
    if renames.is_empty() {
        return;
    }

    // A single global rewrite also covers refs inside the hoisted bodies
    // themselves (including self-referential targets), since by now those
    // bodies live under components.schemas.
    walk_refs(doc, WalkMode::Keywords, &mut |reference: &mut String| {
        if let Some(new_ref) = renames.get(reference.as_str()) {
            *reference = new_ref.clone();
        }
    });
}

/// `#/components/<kind>/<name>/<deeper…>`: a pointer past a whole
/// component, which neither resolver supports directly.
fn is_deep_component_pointer(reference: &str) -> bool {
    let Some(tail) = reference.strip_prefix(COMPONENT_REF_PREFIX) else {
        return false;
    };
    let mut segments = tail.split('/');
    matches!(
        (segments.next(), segments.next(), segments.next()),
        (Some(kind), Some(name), Some(_)) if !kind.is_empty() && !name.is_empty()
    )
}

fn assign_synthetic_names(doc: &Value, pointers: &[String]) -> Vec<(String, String)> {
    let mut taken: HashSet<String> = doc
        .pointer("/components/schemas")
        .and_then(Value::as_object)
        .map(|schemas| schemas.keys().cloned().collect())
        .unwrap_or_default();

    let mut assignments = Vec::new();
    for pointer in pointers {
        // Only subtrees that can stand alone as a schema are hoisted;
        // strings, arrays, and unresolvable pointers are left as-is.
        if !matches!(
            doc.pointer(&pointer[1..]),
            Some(Value::Object(_) | Value::Bool(_))
        ) {
            continue;
        }
        let name = unique_name(synthetic_name_parts(pointer), &mut taken);
        assignments.push((pointer.clone(), name));
    }
    assignments
}

/// Derive a name from the last meaningful pointer segment, suffixed by any
/// trailing structural markers: `…/definitions/accountPrototype` →
/// `AccountPrototype`, `…/parameters/companyId/schema` → `CompanyId`,
/// `…/schemas/Tag/allOf/0` → `TagAllOf0`. The component name is returned
/// separately as a collision-breaking prefix.
fn synthetic_name_parts(pointer: &str) -> (String, Option<String>) {
    let tail = pointer
        .strip_prefix(COMPONENT_REF_PREFIX)
        .unwrap_or(pointer);
    let segments: Vec<String> = tail.split('/').map(decode_pointer_segment).collect();
    let component = segments.get(1).cloned().unwrap_or_default();
    let deeper = segments.get(2..).unwrap_or_default();

    let anchor = deeper
        .iter()
        .rposition(|segment| !is_skipped_segment(segment) && marker_suffix(segment).is_none());

    // When the deeper segments are purely structural the component name
    // itself anchors the name, even if it collides with a marker keyword
    // (a component really can be named `items`).
    let (mut base, suffix_start) = match anchor {
        Some(index) => (pascal_case(&deeper[index]), index + 1),
        None => (pascal_case(&component), 0),
    };
    for segment in deeper.get(suffix_start..).unwrap_or_default() {
        if let Some(suffix) = marker_suffix(segment) {
            base.push_str(&suffix);
        }
    }
    if base.is_empty() {
        base = "Hoisted".to_string();
    }

    let prefix = match anchor {
        Some(_) if !component.is_empty() => Some(pascal_case(&component)),
        _ => None,
    };
    (base, prefix)
}

/// Segments that only describe *where* in a component a pointer landed and
/// carry no naming information of their own. Media types (which contain a
/// decoded `/`) fall in the same bucket.
fn is_skipped_segment(segment: &str) -> bool {
    matches!(
        segment,
        "properties" | "patternProperties" | "definitions" | "$defs" | "schema" | "content"
    ) || segment.contains('/')
}

/// Structural keywords that cannot anchor a name on their own but
/// disambiguate it as a suffix.
fn marker_suffix(segment: &str) -> Option<String> {
    if !segment.is_empty() && segment.bytes().all(|byte| byte.is_ascii_digit()) {
        return Some(segment.to_string());
    }
    match segment {
        "items" => Some("Item".to_string()),
        "allOf"
        | "anyOf"
        | "oneOf"
        | "not"
        | "if"
        | "then"
        | "else"
        | "prefixItems"
        | "additionalProperties"
        | "additionalItems"
        | "contains" => Some(pascal_case(segment)),
        _ => None,
    }
}

fn unique_name((base, prefix): (String, Option<String>), taken: &mut HashSet<String>) -> String {
    if taken.insert(base.clone()) {
        return base;
    }
    if let Some(prefix) = prefix {
        let prefixed = format!("{prefix}{base}");
        if prefixed != base && taken.insert(prefixed.clone()) {
            return prefixed;
        }
    }
    (2..)
        .map(|counter| format!("{base}{counter}"))
        .find(|candidate| taken.insert(candidate.clone()))
        .expect("an unused numeric suffix always exists")
}

fn pascal_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut upper_next = true;
    for character in input.chars() {
        if character.is_alphanumeric() {
            if upper_next {
                out.extend(character.to_uppercase());
                upper_next = false;
            } else {
                out.push(character);
            }
        } else {
            upper_next = true;
        }
    }
    out
}

fn segment_count(pointer: &str) -> usize {
    pointer.split('/').count()
}

fn decode_pointer_segment(segment: &str) -> String {
    // RFC 6901 mandates this order: `~01` is an escaped `~1`, so the `~1`
    // pass must run before `~0` is folded back to `~`.
    segment.replace("~1", "/").replace("~0", "~")
}

fn schemas_map(doc: &mut Value) -> Option<&mut Map<String, Value>> {
    let root = doc.as_object_mut()?;
    let components = root
        .entry("components")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()?;
    components
        .entry("schemas")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
}

/// How to interpret the keys of the object currently being walked. Getting
/// this wrong has real consequences in the corpus: schemas have properties
/// literally named `example`, `links`, or `definitions`, and example
/// literals contain `$ref`-shaped objects that are data, not references.
#[derive(Clone, Copy, PartialEq)]
enum WalkMode {
    /// Keys are spec keywords; `$ref` is a real reference here.
    Keywords,
    /// Keys are user-chosen names (property names, response codes, media
    /// types, …); each value re-enters keyword position.
    Names,
    /// Keys name Example objects, whose bodies are literal payloads (and
    /// whose own `$ref` form must not be retargeted at a schema).
    ExampleNames,
    /// Inside an example/default literal: everything below is data.
    Literal,
}

fn child_mode(mode: WalkMode, key: &str, child: &Value) -> WalkMode {
    match mode {
        WalkMode::Literal | WalkMode::ExampleNames => WalkMode::Literal,
        WalkMode::Names => WalkMode::Keywords,
        WalkMode::Keywords => match key {
            // Literal payload keywords; their content is data even when it
            // happens to be `$ref`-shaped. `default` as a response code
            // is unaffected: it sits under `responses`, a Names map.
            "example" | "default" | "const" | "enum" => WalkMode::Literal,
            // Media-type/parameter `examples` maps name → Example object,
            // while a 3.1 schema `examples` is an array of literal values.
            "examples" => {
                if child.is_array() {
                    WalkMode::Literal
                } else {
                    WalkMode::ExampleNames
                }
            }
            "properties" | "patternProperties" | "definitions" | "$defs" | "paths" | "webhooks"
            | "callbacks" | "responses" | "headers" | "links" | "content" | "encoding"
            | "variables" | "schemas" | "requestBodies" | "securitySchemes" => WalkMode::Names,
            // A keyword map in `components`, but a list of Parameter
            // objects in operations and path items.
            "parameters" => {
                if child.is_object() {
                    WalkMode::Names
                } else {
                    WalkMode::Keywords
                }
            }
            _ => WalkMode::Keywords,
        },
    }
}

/// Visit every `$ref` string in reference position, skipping example
/// literals. Used both to collect deep pointers and to rewrite them.
fn walk_refs(value: &mut Value, mode: WalkMode, visit: &mut dyn FnMut(&mut String)) {
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if mode == WalkMode::Keywords
                    && key == REF_KEY
                    && let Value::String(reference) = child
                {
                    visit(reference);
                    continue;
                }
                let next = child_mode(mode, key, child);
                walk_refs(child, next, visit);
            }
        }
        Value::Array(items) => {
            // Array elements re-enter keyword position (allOf members,
            // parameter lists), except inside literals where everything
            // stays data.
            let next = if mode == WalkMode::Literal {
                WalkMode::Literal
            } else {
                WalkMode::Keywords
            };
            for item in items {
                walk_refs(item, next, visit);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use serde_json::Value;

    use super::{hoist_deep_pointer_refs, relocate_kind_mismatched_component_refs};

    fn parse(fixture: &str) -> Value {
        serde_json::from_str(fixture).expect("fixture parses")
    }

    #[test]
    fn relocates_response_misfiled_under_request_bodies() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.0.3",
                "paths": {
                    "/things/{id}/data": {
                        "put": {
                            "responses": {
                                "200": {
                                    "$ref": "#/components/requestBodies/ThingDataPutResponse"
                                }
                            }
                        }
                    }
                },
                "components": {
                    "requestBodies": {
                        "ThingDataPutResponse": {
                            "content": {
                                "application/json": {
                                    "schema": {"type": "string"}
                                }
                            }
                        }
                    }
                }
            }
        "##});

        relocate_kind_mismatched_component_refs(&mut doc);

        assert_eq!(
            doc.pointer("/paths/~1things~1{id}~1data/put/responses/200/$ref")
                .and_then(Value::as_str),
            Some("#/components/responses/ThingDataPutResponse"),
        );
        let copied = doc
            .pointer("/components/responses/ThingDataPutResponse")
            .expect("component copied to responses");
        assert_eq!(
            copied.get("description").and_then(Value::as_str),
            Some(""),
            "openapiv3 requires a response description",
        );
        assert!(
            copied
                .pointer("/content/application~1json/schema")
                .is_some(),
            "copy keeps the original body",
        );
        assert!(
            doc.pointer("/components/requestBodies/ThingDataPutResponse")
                .is_some(),
            "the misfiled original is preserved for correctly-kinded refs",
        );
    }

    #[test]
    fn relocates_request_body_misfiled_under_responses() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.0.3",
                "paths": {
                    "/things": {
                        "post": {
                            "requestBody": {
                                "$ref": "#/components/responses/ThingPostRequest"
                            },
                            "responses": {}
                        }
                    }
                },
                "components": {
                    "responses": {
                        "ThingPostRequest": {
                            "description": "misfiled request body",
                            "headers": {"X-Trace": {"schema": {"type": "string"}}},
                            "links": {},
                            "content": {
                                "application/json": {
                                    "schema": {"type": "object"}
                                }
                            }
                        }
                    }
                }
            }
        "##});

        relocate_kind_mismatched_component_refs(&mut doc);

        assert_eq!(
            doc.pointer("/paths/~1things/post/requestBody/$ref")
                .and_then(Value::as_str),
            Some("#/components/requestBodies/ThingPostRequest"),
        );
        let copied = doc
            .pointer("/components/requestBodies/ThingPostRequest")
            .expect("component copied to requestBodies");
        assert!(
            copied.get("headers").is_none() && copied.get("links").is_none(),
            "response-only fields are stripped from the request-body copy",
        );
        assert!(copied.pointer("/content/application~1json").is_some());
    }

    #[test]
    fn request_body_relocation_leaves_a_parseable_response_original() {
        // The original stays under `components.responses`, where
        // openapiv3::Response requires a description the request-body
        // shaped object lacks; without patching it the whole 3.0 parse
        // fails even though nothing references the leftover anymore.
        let fixture = indoc! {r##"
            {
                "openapi": "3.0.3",
                "info": {"title": "t", "version": "1"},
                "paths": {
                    "/things": {
                        "post": {
                            "operationId": "postThing",
                            "requestBody": {
                                "$ref": "#/components/responses/ThingPostRequest"
                            },
                            "responses": {
                                "201": {"description": "created"}
                            }
                        }
                    }
                },
                "components": {
                    "responses": {
                        "ThingPostRequest": {
                            "content": {
                                "application/json": {
                                    "schema": {"type": "object"}
                                }
                            }
                        }
                    }
                }
            }
        "##};

        let mut doc = parse(fixture);
        relocate_kind_mismatched_component_refs(&mut doc);

        assert_eq!(
            doc.pointer("/components/responses/ThingPostRequest/description")
                .and_then(Value::as_str),
            Some(""),
            "the leftover original must satisfy openapiv3::Response",
        );
        assert_eq!(
            doc.pointer("/paths/~1things/post/requestBody/$ref")
                .and_then(Value::as_str),
            Some("#/components/requestBodies/ThingPostRequest"),
        );

        crate::parse_openapi_str(fixture).expect("repaired document parses end to end");
    }

    #[test]
    fn never_clobbers_existing_component_of_expected_kind() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.0.3",
                "paths": {
                    "/things": {
                        "get": {
                            "responses": {
                                "200": {"$ref": "#/components/requestBodies/Thing"}
                            }
                        }
                    }
                },
                "components": {
                    "responses": {
                        "Thing": {"description": "the real response"}
                    },
                    "requestBodies": {
                        "Thing": {"content": {}}
                    }
                }
            }
        "##});

        relocate_kind_mismatched_component_refs(&mut doc);

        assert_eq!(
            doc.pointer("/components/responses/Thing/description")
                .and_then(Value::as_str),
            Some("the real response"),
        );
        // Without a copy there is nothing to retarget the ref at.
        assert_eq!(
            doc.pointer("/paths/~1things/get/responses/200/$ref")
                .and_then(Value::as_str),
            Some("#/components/requestBodies/Thing"),
        );
    }

    #[test]
    fn hoists_repeated_deep_pointer_to_single_definition() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.1.0",
                "paths": {},
                "components": {
                    "schemas": {
                        "Account": {
                            "definitions": {
                                "accountPrototype": {"type": "object"}
                            },
                            "allOf": [
                                {"$ref": "#/components/schemas/Account/definitions/accountPrototype"}
                            ]
                        },
                        "CreateAccount": {
                            "$ref": "#/components/schemas/Account/definitions/accountPrototype"
                        }
                    }
                }
            }
        "##});

        hoist_deep_pointer_refs(&mut doc);

        let schemas = doc
            .pointer("/components/schemas")
            .and_then(Value::as_object)
            .expect("schemas object");
        assert_eq!(
            schemas.keys().collect::<Vec<_>>(),
            ["Account", "CreateAccount", "AccountPrototype"],
            "both sites share one hoisted definition",
        );
        assert_eq!(
            schemas["AccountPrototype"]
                .get("type")
                .and_then(Value::as_str),
            Some("object"),
        );
        for site in [
            "/components/schemas/Account/allOf/0/$ref",
            "/components/schemas/CreateAccount/$ref",
            "/components/schemas/Account/definitions/accountPrototype/$ref",
        ] {
            assert_eq!(
                doc.pointer(site).and_then(Value::as_str),
                Some("#/components/schemas/AccountPrototype"),
                "site {site} must point at the hoisted definition",
            );
        }
    }

    #[test]
    fn extracts_nested_pointer_before_its_parent() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.1.0",
                "paths": {},
                "components": {
                    "schemas": {
                        "Pool": {
                            "definitions": {
                                "outer": {
                                    "type": "object",
                                    "properties": {
                                        "inner": {"type": "integer", "format": "int32"}
                                    }
                                }
                            }
                        },
                        "UsesOuter": {
                            "$ref": "#/components/schemas/Pool/definitions/outer"
                        },
                        "UsesInner": {
                            "$ref": "#/components/schemas/Pool/definitions/outer/properties/inner"
                        }
                    }
                }
            }
        "##});

        hoist_deep_pointer_refs(&mut doc);

        // The child target keeps its real body instead of being torn out
        // inside the parent's hoisted subtree.
        assert_eq!(
            doc.pointer("/components/schemas/Inner/format")
                .and_then(Value::as_str),
            Some("int32"),
        );
        assert_eq!(
            doc.pointer("/components/schemas/Outer/properties/inner/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/Inner"),
        );
        assert_eq!(
            doc.pointer("/components/schemas/UsesOuter/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/Outer"),
        );
        assert_eq!(
            doc.pointer("/components/schemas/UsesInner/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/Inner"),
        );
    }

    #[test]
    fn decodes_escaped_media_type_segments() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.0.3",
                "paths": {},
                "components": {
                    "responses": {
                        "Conflict": {
                            "description": "conflict",
                            "content": {
                                "application/json": {
                                    "schema": {"type": "object", "title": "conflict body"}
                                }
                            }
                        }
                    },
                    "schemas": {
                        "Wrapper": {
                            "$ref": "#/components/responses/Conflict/content/application~1json/schema"
                        }
                    }
                }
            }
        "##});

        hoist_deep_pointer_refs(&mut doc);

        assert_eq!(
            doc.pointer("/components/schemas/Conflict/title")
                .and_then(Value::as_str),
            Some("conflict body"),
        );
        assert_eq!(
            doc.pointer("/components/responses/Conflict/content/application~1json/schema/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/Conflict"),
        );
        assert_eq!(
            doc.pointer("/components/schemas/Wrapper/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/Conflict"),
        );
    }

    #[test]
    fn ignores_refs_inside_example_literals() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.0.3",
                "paths": {},
                "components": {
                    "schemas": {
                        "Pool": {
                            "definitions": {
                                "deep": {"type": "integer"}
                            }
                        },
                        "Documented": {
                            "type": "object",
                            "example": {
                                "$ref": "#/components/schemas/Pool/definitions/deep"
                            }
                        },
                        "Uses": {
                            "$ref": "#/components/schemas/Pool/definitions/deep"
                        }
                    },
                    "examples": {
                        "payload": {
                            "value": {
                                "$ref": "#/components/schemas/Pool/definitions/deep"
                            }
                        }
                    }
                }
            }
        "##});

        hoist_deep_pointer_refs(&mut doc);

        // The real site is rewritten…
        assert_eq!(
            doc.pointer("/components/schemas/Uses/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/Deep"),
        );
        // …while example payloads keep their literal `$ref`-shaped data.
        assert_eq!(
            doc.pointer("/components/schemas/Documented/example/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/Pool/definitions/deep"),
        );
        assert_eq!(
            doc.pointer("/components/examples/payload/value/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/Pool/definitions/deep"),
        );
    }

    #[test]
    fn example_only_deep_pointers_are_not_hoisted() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.0.3",
                "paths": {},
                "components": {
                    "examples": {
                        "payload": {"value": {"kind": "demo"}}
                    },
                    "schemas": {
                        "Documented": {
                            "type": "object",
                            "example": {"$ref": "#/components/examples/payload/value"}
                        }
                    }
                }
            }
        "##});

        let before = doc.clone();
        hoist_deep_pointer_refs(&mut doc);
        assert_eq!(
            doc, before,
            "data-only refs must leave the document untouched"
        );
    }

    #[test]
    fn leaves_unresolvable_pointers_untouched() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.0.3",
                "paths": {},
                "components": {
                    "schemas": {
                        "Uses": {
                            "$ref": "#/components/schemas/Missing/definitions/gone"
                        }
                    }
                }
            }
        "##});

        let before = doc.clone();
        hoist_deep_pointer_refs(&mut doc);
        assert_eq!(
            doc, before,
            "dangling deep pointers are reported by the existing machinery",
        );
    }

    #[test]
    fn properties_named_like_keywords_are_not_misclassified() {
        // `definitions` and `example` here are property names, not the
        // draft-04 keyword block or an example literal; the ref nested
        // under the `example` property's schema is a real reference.
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.0.3",
                "paths": {},
                "components": {
                    "schemas": {
                        "Pool": {
                            "definitions": {
                                "deep": {"type": "integer"}
                            }
                        },
                        "Weird": {
                            "type": "object",
                            "properties": {
                                "definitions": {"type": "string"},
                                "example": {
                                    "type": "object",
                                    "properties": {
                                        "link": {
                                            "$ref": "#/components/schemas/Pool/definitions/deep"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "##});

        hoist_deep_pointer_refs(&mut doc);

        assert_eq!(
            doc.pointer("/components/schemas/Weird/properties/example/properties/link/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/Deep"),
            "a ref under a property named `example` is still a reference",
        );
        assert_eq!(
            doc.pointer("/components/schemas/Weird/properties/definitions/type")
                .and_then(Value::as_str),
            Some("string"),
            "a property named `definitions` is left alone",
        );
    }

    #[test]
    fn collision_falls_back_to_component_prefixed_name() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.1.0",
                "paths": {},
                "components": {
                    "schemas": {
                        "Bill": {
                            "definitions": {"recordRef": {"type": "string"}}
                        },
                        "Invoice": {
                            "definitions": {"recordRef": {"type": "integer"}}
                        },
                        "UsesBill": {
                            "$ref": "#/components/schemas/Bill/definitions/recordRef"
                        },
                        "UsesInvoice": {
                            "$ref": "#/components/schemas/Invoice/definitions/recordRef"
                        }
                    }
                }
            }
        "##});

        hoist_deep_pointer_refs(&mut doc);

        assert_eq!(
            doc.pointer("/components/schemas/RecordRef/type")
                .and_then(Value::as_str),
            Some("string"),
        );
        assert_eq!(
            doc.pointer("/components/schemas/InvoiceRecordRef/type")
                .and_then(Value::as_str),
            Some("integer"),
        );
        assert_eq!(
            doc.pointer("/components/schemas/UsesInvoice/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/InvoiceRecordRef"),
        );
    }

    #[test]
    fn hoists_self_referential_target() {
        let mut doc = parse(indoc! {r##"
            {
                "openapi": "3.1.0",
                "paths": {},
                "components": {
                    "schemas": {
                        "PushOption": {
                            "definitions": {
                                "pushOptionProperty": {
                                    "type": "object",
                                    "properties": {
                                        "child": {
                                            "$ref": "#/components/schemas/PushOption/definitions/pushOptionProperty"
                                        }
                                    }
                                }
                            }
                        },
                        "Uses": {
                            "$ref": "#/components/schemas/PushOption/definitions/pushOptionProperty"
                        }
                    }
                }
            }
        "##});

        hoist_deep_pointer_refs(&mut doc);

        assert_eq!(
            doc.pointer("/components/schemas/PushOptionProperty/properties/child/$ref")
                .and_then(Value::as_str),
            Some("#/components/schemas/PushOptionProperty"),
            "the recursive ref inside the hoisted body is rewritten too",
        );
    }
}
