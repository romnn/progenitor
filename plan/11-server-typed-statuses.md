# 11 — Status-typed responses: no runtime contract validation in generated code

Request (Roman, 2026-07-24): the generated client gets fully typed errors, so the server must too —
wire parity checked at compile time. An implementor must not be able to return a status/payload
combination the spec does not declare; "compiles, then 500s at runtime because the status guard
fired" is not acceptable. Remove runtime contract validation from generated code entirely.

**Principle**: generated code may fail at runtime only on (a) untrusted wire input (extraction —
parse, don't validate, at the boundary) and (b) genuine infrastructure failure
(`ServerError::Internal` → 500). Every value the *implementor* can construct must type-check into a
spec-conformant wire response. Status is never a free `StatusCode` unless the spec itself declares
it free (`default` responses) — and where the spec constrains it to a class (`4XX`), the constraint
is enforced at the implementor's construction site with an explicit fallible constructor, never
silently at respond time.

Status: **PLANNED**. Grounded in branch `feat/openapi3.1-support` at `c38e41d`. Supersedes plan
07's "status resolution contract" / "status-validation matrix" (§6.4) — those runtime rules are
replaced by types, and this plan deletes their implementation.

## 1. Current state (measured) — where the type system currently gives up

- `ServerError::Api { status: StatusCode, body: E }`
  (`crates/progenitor-server/src/error.rs`): status is free runtime data on every error return.
- `Response<T>::with_status` (`crates/progenitor-server/src/response.rs:34`): free status override
  on every success return; `into_parts` (`response.rs:76`) yields `Option<StatusCode>`.
- The only defense is a **generated runtime guard**: `response_status_guard` /
  `response_status_predicate` / `StatusPredicate` (`crates/progenitor-impl/src/server.rs:986-1100`)
  emit an `if !(declared)` check into every responder (call sites `server.rs:387-430`) that turns
  an undeclared status into `respond::internal` — a 500 with "operation `{id}` returned undeclared
  {side} status". This is exactly "compiles, fails at runtime".
- The typed approach already half-exists: server-side synth enums carry a status per variant
  (`server.rs:484-530` — `Code(n)` implied, `Range`/`Default` carry a `status` field) with a
  `From<{Synth}> for ServerError<_>` impl deriving the status (`server.rs:498-513`). But synth
  enums are generated **only when response kinds differ**
  (`crates/progenitor-impl/src/operations/responses.rs:179`); two same-kind declared statuses
  (404 → `Error`, 409 → `Error`) fall back to the free-status `Api` path. And even synth values
  route through `Api { status, body }`, so the unchecked constructor remains reachable.
- One stray generated panic: `StatusCode::from_u16(#default_status).unwrap()` (`server.rs:394`).
- The client (`operations/responses.rs` arm generation) matches typed errors by declared status —
  parity with the server today rests on the runtime guard mirroring the client classifier, not on
  types.

## 2. Design

Make the per-response-item enum the **only** representation, for both sides, whenever status is not
uniquely implied — and remove free status everywhere else.

### 2.1 Runtime types (`progenitor-server`)

```rust
// error.rs — `Api` no longer carries a status; all response data lives in E.
pub enum ServerError<E> {
    Api(E),
    Internal(BoxError),
    Response(axum::response::Response),
}
// DELETED: ServerError::api(status, body). From<E> impls (generated) remain the
// construction path: Err(NotFound(body).into()).

// response.rs — success wrapper is body + headers only.
pub struct Response<T> { message: T, headers: HeaderMap }
// DELETED: with_status, the status field, and the Option<StatusCode> in into_parts.

// status.rs (new) — a status constrained to one class (a spec `4XX` range).
pub struct ClassStatus<const CLASS: u16>(StatusCode);
impl<const CLASS: u16> ClassStatus<CLASS> {
    /// `None` unless `status.as_u16() / 100 == CLASS`. The implementor handles
    /// the `Option` at construction — respond time is infallible.
    pub fn new(status: StatusCode) -> Option<Self>;
    pub fn get(self) -> StatusCode;
}
// Plus per-class associated consts for the standard codes (e.g.
// ClassStatus::<4>::NOT_FOUND) so common construction is infallible.
```

### 2.2 The typing rule, per response side of each operation

Let `items` be the side's declared response items (`extract_responses`):

| declared items | trait-visible type | status at respond time |
|---|---|---|
| exactly one `Code(n)` | the payload type itself | generation-time constant |
| anything else (≥2 items, or any `Range`/`Default`) | a generated per-op enum, one variant per item | derived from the variant, infallibly |

Variant shapes (generalizing today's synth definitions, `server.rs:531-560`):

- `Code(n)` → `Status404(payload)` / named variant — status implied by the variant.
- `Range(r)` → `Status4xx { status: ClassStatus<4>, body: payload }` — class-checked at
  construction.
- `Default` → `Default { status: StatusCode, body: payload }` — free **because the spec says any
  status**; there is no invariant to check, so no check exists.

This replaces the "synth only on differing kinds" trigger (`responses.rs:179`) with "enum whenever
status is not uniquely implied"; same-kind multi-status ops get the same treatment as mixed-kind
ops. Existing synth naming/machinery (`server_synth_ident`, variant definitions, per-variant
`From`/helper constructors) is reused, not duplicated — the *kind*-driven and *status*-driven cases
emit through one path.

### 2.3 Responders become total functions

With status derived from the type, the responders (`server.rs:377-430`) reduce to: match the
variant (or use the generation-time constant), emit via the existing `respond::json/empty/bytes`
helpers. **Delete**: `response_status_guard`, `response_status_predicate`, `StatusPredicate`,
`invalid_condition` (`server.rs:986-1100`) and both guard call sites. The plan 07 §6.4
status-validation matrix and the classifier's server-side *validation* role disappear; the
classifier's client-side arm generation is untouched.

The stray `from_u16(..).unwrap()` (`server.rs:394`): validate declared status codes once at
generation time (reject out-of-range at generation, like other spec errors) and emit
`http::StatusCode` named consts for standard codes, `from_u16` + `expect` with a
generation-guaranteed-valid comment otherwise — no reachable panic.

### 2.4 Wire parity, by construction

Client match arms and server enum variants are now both generated from the same `items` list. A
spec change regenerates both sides; a server impl that no longer covers a removed variant, or
doesn't handle an added one, **fails to compile** (exhaustive enum construction on the impl side,
exhaustive match in the generated responder). No shared runtime classifier contract is needed for
correctness anymore — the conformance round-trip (petstore-31) remains as the end-to-end proof.

## 3. Implementation steps

1. **Runtime** (`progenitor-server`): `ClassStatus` (with tests: class check, consts), remove
   `ServerError::api` + the `Api` status field, remove `Response::with_status`/status field,
   adjust `into_parts` and `respond::*` signatures as needed. This intentionally breaks compile
   for generated code — the error list drives step 2.
2. **Generator** (`server.rs`): implement the §2.2 table — extend enum generation to the
   status-driven case, reuse synth variant/`From` machinery, derive statuses in responders, delete
   the guard machinery (`server.rs:986-1100`) and `with_status` handling, apply the §2.3
   `from_u16` cleanup. Client/httpmock/cli backends are untouched (client arms already status-
   typed).
3. **Fixtures + example**: update `crates/example-server` (constructions change:
   `ServerError::api(BAD_REQUEST, e)` → `Err({Op}Error::Status400(e).into())`); regenerate golden
   fixtures (`crates/progenitor-impl/tests/output/src/server_gen_*.rs`) — review the diff for
   exactly: enums appearing, guards disappearing, no other drift.
4. **Conformance** (`conformance/petstore-31`): extend the round-trip with a same-kind
   multi-status operation (e.g. 404 and 409 both → `Error`) asserting each variant arrives at the
   client as the right typed error; a `default`-response op asserting the carried status round-
   trips; and — as a compile-time assertion — a doctest/`compile_fail` case showing an undeclared
   status is unrepresentable (no `StatusCode` parameter exists to abuse).
5. **Docs**: mark plan 07 §6.4's status matrix as superseded by this plan (one-line note at the
   top of that section, content left for history).

## 4. Acceptance criteria

- No generated responder contains a status check, and `grep response_status_guard` returns
  nothing: an implementor cannot express an undeclared status/payload combination for any
  operation in the conformance corpus — the API surface simply has no place to put one
  (`compile_fail` case pinned).
- The only fallible paths in generated server code are extraction rejections and
  `ServerError::Internal`; `ClassStatus::new` is the only implementor-facing fallible constructor
  and only appears for spec-declared range responses.
- petstore-31 round-trip passes including the multi-status and default-response ops; every corpus
  spec still generates and compiles (same-kind multi-status ops across the corpus now emit enums —
  the compile tier is the regression gate).
- `Response<T>` and `ServerError<E>` have no status-typed field other than those carried inside
  generated variants.

## 5. Non-goals

- Typed **headers** on responses (headers remain free `HeaderMap` data — the spec's response
  headers are documentation-grade today; typing them is a separate decision).
- Client-side changes — client error typing is already status-driven; parity comes from shared IR.
- Per-operation typed extraction errors (plan 07 §6.5 follow-up; unchanged by this plan).

## 6. Sequencing with plans 09/10

Land **11 → 10 → 09**. Plan 10's hook threads `&T` into the responders this plan rewrites — doing
11 first means 10 touches the final responder shape once; and with the guards gone, plan 10's
`RejectionKind::StatusContract` surface never needs to exist (plan 10 has been amended
accordingly). Plan 09's multipart is request-side and orthogonal, but lands last so its generated
route code uses both the final rejection funnel and the final responder shape.

## 7. Design rationale (architecture notes)

- **Make illegal states unrepresentable, not runtime-rejected**: the guard was a runtime re-check
  of an invariant the generator fully knows at generation time — the textbook case for moving an
  invariant into types. The deleted guard machinery (~120 lines + per-op generated checks) is the
  rent the types pay back immediately.
- **`Default` carries free status without validation** because the spec grants that freedom —
  adding a check there would invent a contract the spec doesn't state.
- **`Range` uses a fallible constructor at the construction site** rather than a 100-variant enum
  or respond-time failure: the invariant is real but value-dependent; surfacing the `Option` where
  the implementor chooses the status is honest, local, and leaves respond time total.
- **One enum mechanism for kind-driven and status-driven cases**: generalizing synth generation
  avoids two parallel enum generators that would drift; the trigger condition changes, the emitted
  shape does not.
