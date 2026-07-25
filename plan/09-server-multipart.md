# 09 — Typed multipart bodies for server + client generation

Request (Roman, 2026-07-24): real `multipart/form-data` support. A multipart operation must get a
typed trait method (named fields, file parts) on the server and a typed request builder on the
client — implementors and callers never touch boundaries, part iteration, or content-type headers.
Dropshot proves in-process multipart is viable, but its `MultipartBody` hands the handler a raw
`multer::Multipart`; that violates this fork's #1 server rule ("implementors must not touch
serialization or header parsing", plan 07 §1), so we generate typed parts instead.

Status: **IMPLEMENTED** (2026-07-25). Landed as designed with two refinements:
`Parts::required_text`/`optional_text` are generic over `T: FromStr` (parsing
lives in the runtime like `required_header`, instead of §2.2's `String`-returning
accessors plus generated parse blocks), and a text property whose schema typify
cannot represent degrades to `String` rather than failing generation.

## 1. Current state (measured)

- `BodyContentType::from_str` classifies `multipart/form-data` as `Raw("multipart/form-data")`
  (`crates/progenitor-impl/src/operation.rs:160-174`; variants at `operation.rs:133-144`).
- Lowering maps `Raw` to `OperationParameterType::RawBody`
  (`crates/progenitor-impl/src/operations/lower.rs:527`); multi-media bodies prefer JSON, else the
  first declared variant (`lower.rs:454-471`, referencing progenitor#418).
- **Server**: `Raw` bodies fall through `body_extractor` to the `Bytes` passthrough
  (`crates/progenitor-impl/src/server.rs:938-968`) — the trait method receives `bytes::Bytes` and
  the implementor must hand-parse multipart. Only upgrade responses and deepObject queries are
  501-stubbed (`server.rs:187-215`); multipart is *not* stubbed, just untyped.
- **Client**: a `Raw` body becomes a `reqwest::Body` parameter with the content type set verbatim.
  For multipart this is a latent bug: the header is emitted **without a boundary parameter**, so no
  server can parse the result. Typed multipart fixes this as a side effect (reqwest generates the
  boundary).
- Workspace: `axum` has `default-features = false` (root `Cargo.toml:60`) — the `multipart` feature
  is not enabled anywhere. `reqwest` has `json`/`query`/`stream` only (`Cargo.toml:93`) — no
  `multipart` feature.

## 2. Design

One new IR concept, one new runtime module, and per-backend emission that follows two existing
precedents: the generated `{Op}Query` struct (server-authored param struct built field-by-field,
`server.rs:267,851`) and the parallel-but-distinct client/server synth enums (plan 07 §6.4 — both
sides derive from the same IR item list, which guarantees wire agreement without sharing a type).

### 2.1 IR: `MultipartSpec` (generator-side single source of truth)

In `operation.rs`:

```rust
pub(crate) enum BodyContentType {
    // existing variants …
    Multipart,                       // base media type "multipart/form-data"
}

pub(crate) struct MultipartSpec {
    pub fields: Vec<MultipartField>,
}

pub(crate) struct MultipartField {
    pub name: String,                // sanitized Rust field name
    pub api_name: String,            // wire part name
    pub description: Option<String>,
    pub kind: MultipartFieldKind,
    pub required: bool,              // from the object schema's `required`
    pub repeated: bool,              // array-of-file → Vec<FilePart>
}

pub(crate) enum MultipartFieldKind {
    File,                            // binary part: filename + content-type + bytes
    Text(TypeId),                    // scalar part parsed from the part's text
}
```

Add `OperationParameterType::Multipart(MultipartSpec)` alongside `Type`/`RawBody`
(`operation.rs:117-120`). The two enums are the seam: every backend (server, client, httpmock, cli)
matches exhaustively on `OperationParameterType`, so the compiler enumerates every site that must
handle the new variant — that error list is the todo list.

**Classification rules** (in `get_body_param`, `lower.rs:444-527`):

- `from_str` returns `Multipart` for base `multipart/form-data`.
- If the media type has an **object schema with properties**, build `MultipartSpec`:
  - A property is a **File** field when its schema (or its array `items` schema, setting
    `repeated`) is a string with 3.0 `format: binary`, or 3.1 `contentEncoding` /
    `contentMediaType`.
  - Every other property is **Text**: lower the property schema through typify; if the resulting
    type lacks `FromStr`, degrade to `String` — the exact rule `owned_field_type` applies to
    path/query/header params (`server.rs:896-935`). Nested objects/arrays therefore degrade to
    `String` carrying the raw part text; that is deliberate v1 coarseness, not an error.
- If there is **no schema or a non-object schema**, keep today's behavior exactly: classify as
  `Raw("multipart/form-data")` → raw passthrough. Generation must never fail on a wild spec.

### 2.2 Runtime: `progenitor-server/src/multipart.rs`

The runtime owns all wire mechanics (multer via axum, part iteration, memory collection); the
generator contributes only names and types. Enable the `multipart` feature on the workspace `axum`
dependency.

```rust
/// One uploaded file part, fully read into memory.
pub struct FilePart {
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub bytes: bytes::Bytes,
}

/// Body extractor: wraps `axum::extract::Multipart`, converting its rejection
/// (missing/invalid content type, malformed body) into our `Rejection`, like
/// the existing `Json`/`Form` extractors (extract.rs:172-208). Lives in
/// `extract.rs` next to them; the parsed-parts logic below lives in
/// `multipart.rs`.
pub struct Multipart(/* axum's extractor */);

/// All parts of a request, collected into memory, keyed by wire part name.
/// Each raw part keeps `filename`, `content_type`, and `bytes` — collection
/// does NOT classify parts as text vs file; the accessor called on a part is
/// the spec's view and decides the interpretation.
pub struct Parts(/* IndexMap<String, Vec<RawPart>> */);

impl Parts {
    pub async fn collect(multipart: Multipart) -> Result<Self, Rejection>;
    // Text accessors decode the part's bytes as UTF-8 (400 on invalid UTF-8).
    pub fn required_text(&mut self, api_name: &str) -> Result<String, Rejection>;
    pub fn optional_text(&mut self, api_name: &str) -> Result<Option<String>, Rejection>;
    // File accessors wrap the raw part verbatim.
    pub fn required_file(&mut self, api_name: &str) -> Result<FilePart, Rejection>;
    pub fn optional_file(&mut self, api_name: &str) -> Result<Option<FilePart>, Rejection>;
    pub fn repeated_file(&mut self, api_name: &str) -> Result<Vec<FilePart>, Rejection>;
}
```

There is deliberately no wire-side text-vs-file classification (an earlier sketch guessed from
`filename`/content-type — an avoidable heuristic): the generated accessors already encode which
fields the spec declares as files, so the accessor is the single source of interpretation. Failure
cases are deterministic: missing required part and invalid UTF-8 in a text accessor are
`Rejection::bad_request` naming the part. **Unknown parts are ignored** — forward compatibility,
same posture as serde's default unknown-field tolerance. Total body size is bounded by the caller's
`DefaultBodyLimit` layer, not by this module (documented; see non-goals).

### 2.3 Server generation

- `body_extractor` (`server.rs:938-968`) gains a `Multipart` arm: extractor arg
  `__progenitor_multipart: ::progenitor_server::Multipart` (a `FromRequest` body extractor — stays
  last, like `Json`/`Bytes`).
- `server_op` emits, per multipart operation (module item next to `{Op}Query`,
  `server.rs:267,296`):

```rust
#[derive(Debug, Clone)]
pub struct {Op}MultipartBody {
    pub note: Option<String>,          // Text fields: FromStr-parsed owned types
    pub file: ::progenitor_server::multipart::FilePart,   // File fields
}
```

- The generated route body (before `let message = …`, `server.rs:310-321`) collects and binds,
  early-returning `Rejection`s exactly like the header lets (`server.rs:777-830`; once plan 10
  lands these route through `render_rejection` — see sequencing):

```rust
let mut __progenitor_parts =
    match ::progenitor_server::multipart::Parts::collect(__progenitor_multipart).await {
        Ok(parts) => parts,
        Err(rejection) => return /* rejection path */,
    };
let note = /* optional_text("note") + FromStr parse for non-String types, same early-return */;
let file = /* required_file("file") */;
let message = {Op}Request { body: {Op}MultipartBody { note, file }, /* other params */ };
```

The request struct's `body` field type becomes `{Op}MultipartBody` (request struct at
`server.rs:290-294`).

### 2.4 Client generation

Emit a **parallel, client-side** `{Op}MultipartBody` struct (same field layout;
`FilePart { filename, content_type, bytes }` defined in `progenitor-client` — the client must not
depend on `progenitor-server`). The generated method takes it instead of `reqwest::Body` and builds
`reqwest::multipart::Form`: text fields via `.text(api_name, value.to_string())`, file fields via
`Part::bytes(..).file_name(..)` + `.mime_str(..)` when set. reqwest owns the boundary and the
`content-type` header — delete the verbatim-header path for multipart. Enable reqwest's
`multipart` feature (root `Cargo.toml:93`). Ops that still classify as `Raw` (schema-less
multipart) keep the existing `reqwest::Body` signature unchanged.

httpmock/cli backends: `Multipart(_)` matches like `RawBody` today (mock body assertions on
multipart are a non-goal; the cli skips the op with the existing skip mechanism if it cannot
represent the body).

## 3. Implementation steps

1. **IR** (`operation.rs`, `operations/lower.rs`): add `BodyContentType::Multipart`,
   `OperationParameterType::Multipart(MultipartSpec)`, classification in `get_body_param` with the
   schema-less fallback to `Raw`. Fix every non-exhaustive-match compile error this creates —
   backends default-treating `Multipart` like `RawBody` is the correct intermediate state; the
   build must stay green after this step alone.
2. **Runtime** (`progenitor-server`): `multipart.rs` (FilePart, Parts) + the `Multipart` extractor
   in `extract.rs`; axum `multipart` feature; unit tests for classification, required/optional/
   repeated accessors, unknown-part tolerance, and rejection messages (follow the existing
   extract.rs test style, `extract.rs:353-450`).
3. **Server generator** (`server.rs`): `body_extractor` arm, `{Op}MultipartBody` emission, route
   binding block. Golden fixture: extend the server-generation fixture spec with a multipart op and
   update `crates/progenitor-impl/tests/output/src/server_gen_*.rs` via the existing golden-update
   flow.
4. **Client generator**: client `FilePart`, client `{Op}MultipartBody`, `reqwest::multipart::Form`
   emission; reqwest `multipart` feature.
5. **Conformance round-trip** (`conformance/petstore-31`, `spec.toml` already has `server = true`):
   add `POST /pets/{petId}/photo` — required binary `file`, optional text `note`, a typed 201
   response. Round-trip test: generated client uploads → trait impl asserts typed fields → assert
   response; plus missing-required-part → 400 and unknown-extra-part → ignored (assert via the
   generated client where decodable, raw HTTP otherwise, per plan 07 §6.5's rejection caveat).
6. **Corpus sweep**: regenerate the wild corpus. Ops that previously compiled as raw multipart now
   change signatures — the 60-spec compile tier is the regression gate; fix or (schema-less cases)
   confirm they still fall back to `Raw`.

## 4. Acceptance criteria

- A multipart operation with an object schema yields a trait method whose request carries a typed
  `{Op}MultipartBody`; the implementor never sees boundaries, parts, or raw bytes (unless a field
  is a `FilePart`, whose `bytes` are the file content only).
- The generated client for the same spec uploads via `reqwest::multipart` with a correct boundary;
  the petstore-31 client↔server round-trip passes.
- Schema-less multipart bodies behave exactly as before (raw passthrough); zero corpus specs
  regress from generating to failing.
- Missing required part / type-mismatched part → deterministic 400 `Rejection` naming the part;
  unknown parts ignored.

## 5. Non-goals (v1) and sequencing

- **Streaming parts**: `FilePart.bytes` is in-memory `Bytes`; size is bounded by the caller's
  `DefaultBodyLimit`. Streaming is a follow-up with its own design (same posture as plan 07's D-raw).
- **JSON-object parts / `encoding` interactions**: non-scalar text fields degrade to `String` of
  the raw part text. No per-part `headers`/`style` handling.
- **Per-part size limits** and multipart **response** bodies.
- **httpmock typed multipart assertions**; the mock treats the body as opaque.
- Sequencing: land order across the server plans is **11 → 10 → 09** (plan 11 §6). With 10 in
  place first, step 3's early-returns call `render_rejection` from day one, and the route code is
  written against the post-11 responder shape — no migration sweeps.

## 6. Design rationale (architecture notes)

- **Seam placement**: the runtime owns wire mechanics (multer, memory collection, part
  classification); the generator owns spec knowledge (names, kinds, requiredness) and emits only
  thin binding code. Neither knows the other's internals beyond `Parts`' accessor surface.
- **Typed fields over dropshot-style raw `multer::Multipart` passthrough**: rejected the
  passthrough because the fork's stated contract is that implementors never parse; a raw handle
  would also make the spec's field list decorative (drift surface).
- **Parallel client/server structs over a shared type**: follows the established synth-enum
  precedent; sharing would force a dependency between `progenitor-client` and `progenitor-server`
  for one struct.
- **Descriptor-driven runtime + generated binding over fully generated parse loops**: the multer
  loop is identical for every operation — generating it per-op would be N copies of wire code;
  binding (names/types) is the only per-op part, so only that is generated.
- **Degrade, never fail**: schema-less → `Raw`, non-scalar text → `String`. Wild-corpus
  generation keeps its "one exotic endpoint never sinks the client" guarantee
  (`operation.rs:138-143`).
