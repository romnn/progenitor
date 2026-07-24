# 10 — Custom rejection envelope: one error shape for the whole server

Request (Roman, 2026-07-24): a public API must speak **one** error shape. Today the generated
server has two: declared errors go through the typed `ServerError::Api` path and render whatever
the spec says, but extraction failures (malformed body, bad query/path/header) and internal 500s
render a runtime-standard plain-text body that the service author cannot influence. Add a hook so
an implementation can render every non-`Api` error through its own envelope (e.g. a partner API's
`ApiError` JSON), with the current behavior as the default.

Plan 11 (typed statuses) does **not** make this obsolete: it types what the *implementor* returns,
while extraction failures happen before the trait method is ever called — malformed wire bytes
exist no matter how well-typed the handler is, and the spec cannot declare away a 400 for an
operation that documents no error response. A service-wide renderer is therefore the necessary
base layer; mapping rejections into per-operation *declared* error types remains the optional
refinement tracked in §5.

Status: **IMPLEMENTED** (2026-07-24). Landed after plan 11's typed responder
rewrite, so every extraction and internal-error funnel targets the final
responder shape.

## 1. Current state (measured)

Every non-`Api` error surface and where it renders today (post-plan-11 tree — the status-guard 500s
plan 07 §6.4 defined are removed by plan 11, not funneled here):

| surface | where | rendering today |
|---|---|---|
| body/query/path extractor failure | extractor args short-circuit **before** the route body; axum converts via `IntoResponse for Rejection` (`crates/progenitor-server/src/extract.rs:72-76`) | `(status, message)` plain text |
| header/cookie parse failure | generated in-route early returns (`crates/progenitor-impl/src/server.rs:777-830`) | `IntoResponse::into_response(rejection)` — same plain text |
| `ServerError::Internal` | generated responder arm (`server.rs:337-339`) → `respond::internal` (`crates/progenitor-server/src/response.rs:158`, drops the error value) | plain 500 |
| `Rejection` itself | `extract.rs:27-70` — `{ status: StatusCode, message: String }`, constructors `new` / `bad_request` / `unsupported_media_type` | no machine-readable category |

The responder is a static fn — `Self::{op}_respond(result)` (`server.rs:320,323-343`) — with no
access to the implementation, so today nothing per-service *can* influence these bodies.

## 2. Design

Two moves: make `Rejection` carry a machine-readable category, and give the generated service
trait one provided rendering method that every non-`Api` surface funnels through. The hook lives
**on the trait** because that is the seam the implementor already owns — no state plumbing, no
layer, and the default preserves today's bytes exactly.

### 2.1 `Rejection` gains a `kind`

In `extract.rs`:

```rust
#[non_exhaustive]                       // future surfaces must not break impls
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionKind {
    InvalidBody,            // malformed JSON/form/text, read failure
    UnsupportedMediaType,   // wrong/missing content-type
    InvalidQuery,
    InvalidPath,
    MissingHeader,
    InvalidHeader,
    MissingCookie,
    InvalidCookie,
    MissingPart,            // multipart (plan 09)
    InvalidPart,            // multipart (plan 09)
    Internal,               // ServerError::Internal (500)
}

pub struct Rejection { status: StatusCode, kind: RejectionKind, message: String }
impl Rejection {
    pub fn kind(&self) -> RejectionKind;   // status()/message() exist (extract.rs:53-61)
}
```

Constructor changes: `new(status, kind, message)`; the convenience constructors
(`bad_request`, `unsupported_media_type`) either gain a `kind` parameter or are replaced by
per-kind constructors (`invalid_body(msg)`, `missing_header(msg)`, …) — prefer per-kind
constructors: call sites read better and the kind can never disagree with the site. Update every
constructor call site in `extract.rs` (extractors at `:128-256`, header/cookie helpers at
`:260-331`) and add an `internal()` constructor (fixed 500 status) for §2.3.
`IntoResponse for Rejection` (`extract.rs:72-76`) is unchanged — it *is* the default rendering.

A renderer matches on `kind` (with a required wildcard arm, since the enum is `#[non_exhaustive]`)
to map onto its own error-code set; `status()`/`message()` carry the rest. No string parsing.

### 2.2 The trait hook

`server_body` (`crates/progenitor-impl/src/server.rs:116-129`) adds one provided method to the
generated trait:

```rust
pub trait {Api}: Send + Sync + 'static {
    // … generated operation methods …

    /// Render a request-level failure (extraction rejection, internal error,
    /// contract violation) as the HTTP response. The default preserves the
    /// runtime-standard body. Override to emit your API's error envelope; the
    /// generated adapter restores the rejection's status after rendering.
    fn render_rejection(
        &self,
        rejection: ::progenitor_server::Rejection,
    ) -> axum::response::Response {
        ::progenitor_server::codegen::axum::response::IntoResponse::into_response(rejection)
    }
}
```

Sync, non-async (rendering is pure formatting; keeps it out of `async_trait` boxing), and a
provided default so every existing implementation keeps compiling byte-identically.

The hook shares the trait method namespace with operation IDs. A server operation that sanitizes to
`render_rejection` therefore receives the first free numeric suffix, using the same deterministic
collision style as other generated operation names.

### 2.3 Funnel all three surfaces through the hook

All in `server_op` / its helpers:

1. **Extractor args → `Result`-wrapped.** axum implements `FromRequestParts`/`FromRequest` for
   `Result<T, T::Rejection>`, and our extractors' `Rejection` type *is*
   `progenitor_server::Rejection` — so each generated extractor arg
   (`server.rs:938-968` for the body; `collect_params` for path/query) becomes
   `Result<Extractor, Rejection>` and no longer short-circuits before the handler. The route body
   unwraps first thing:

   ```rust
   let __progenitor_body = match __progenitor_body {
       Ok(::progenitor_server::Json(v)) => v,
       Err(rejection) => {
           return <T as {Api}>::render_rejection(&__progenitor_inner, rejection);
       }
   };
   ```

   `State` and `Metadata` stay bare (infallible). Extractor order is unchanged (body last).
2. **Header/cookie early returns** (`server.rs:777-830`): replace
   `return IntoResponse::into_response(rejection)` with the same
   `return <T as {Api}>::render_rejection(&__progenitor_inner, rejection)`.
3. **Responder** gains the implementation: signature becomes
   `fn {op}_respond(inner: &T, result: …) -> Response` and the call site
   (`server.rs:320`) passes `&__progenitor_inner`. The `Internal` arm becomes: log via the
   runtime (keep the current logging in `respond::internal`, split into
   `respond::log_internal(&error)` so the error value is still recorded), then
   `inner.render_rejection(Rejection::internal())`. The error value itself is never given to the
   renderer — it must not leak into response bodies, which is the existing `respond::internal`
   posture (`response.rs:158`).

The adapter records `rejection.status()` before invoking the hook and restores it on the returned
response. Body and header customization therefore cannot accidentally violate the rejection's
status contract.

`ServerError::Api` and `ServerError::Response` arms are untouched — declared errors and the escape
hatch already belong to the implementor.

### 2.4 Interop caveat (document, don't solve)

The generated *client* decodes error bodies only for statuses the spec declares (plan 07 §6.5's
rejection caveat). A custom envelope makes 4xx rejection bodies decodable by partners **when the
spec declares those statuses** (e.g. a documented 400 with the envelope schema); otherwise clients
surface them as unexpected-response, same as today. Recommend spec authors declare a default/400
error response using their envelope schema — that is exactly the partner-API use case driving this
feature.

## 3. Implementation steps

1. **Runtime** (`progenitor-server/src/extract.rs`): `RejectionKind`, per-kind constructors,
   migrate all constructor call sites, add `internal()`; split
   `respond::internal` into log + render-default halves (`response.rs:158`). Unit tests: each
   extractor's rejection carries the right kind (extend the existing test module,
   `extract.rs:353-450`).
2. **Generator** (`server.rs`): emit the provided trait method; `Result`-wrap extractor args and
   unwrap-or-render in the route body; convert header/cookie early returns; thread `&T` into the
   responders; route the `Internal` path through the hook. Golden fixture update
   (`crates/progenitor-impl/tests/output/src/server_gen_*.rs`).
3. **Example** (`crates/example-server`): override `render_rejection` with a small JSON envelope
   (`{"code": …, "message": …}`, kind-mapped) as living documentation.
4. **Conformance** (`conformance/petstore-31`): two round-trip additions — (a) default: malformed
   JSON body → assert today's plain-text 400 via raw HTTP (unchanged behavior pinned); (b)
   override: an impl with a custom envelope → assert the envelope + correct status for a body
   rejection, invalid query/path values, missing required header/cookie values, and an `Internal`
   return. The override deliberately returns the wrong status so the adapter's status enforcement
   is also pinned.

## 4. Acceptance criteria

- An implementation overriding nothing produces byte-identical responses to today (pinned by the
  default-path conformance test and the golden diff being additive-only around the funnel points).
- An implementation overriding `render_rejection` controls the body of body/query/path/header/
  cookie extraction failures and `ServerError::Internal` — verified by the round-trip test.
- The generated adapter preserves `Rejection::status()` even when an override returns a different
  response status.
- An operation ID that sanitizes to `render_rejection` still generates a compilable client and
  server.
- `Rejection::kind()` is sufficient to map every surface onto a custom error-code set without
  parsing `message`.
- The internal error value is logged but never reaches the renderer.

## 5. Non-goals

- **Async rendering** and renderer access to request context (the rejection carries what it
  carries; a renderer needing the request should use the `ServerError::Response` escape hatch at
  the operation level).
- **Per-operation typed extraction errors** (mapping a rejection into the operation's declared
  error *type*) — remains the tracked follow-up from plan 07 §6.5; this hook is the service-wide
  80% solution and does not preclude it.
- Changing the status codes themselves — the hook renders bodies; the adapter restores the
  runtime's `rejection.status()` so a renderer cannot desync the client's status-based decode arms
  (statuses are otherwise fully typed per plan 11).

## 6. Design rationale (architecture notes)

- **Hook on the trait, not on `Server`/`Router` or a tower layer**: the trait is the seam the
  implementor already owns, works identically for `into_router()` power users who never touch the
  transport builder, is type-checked (no `Arc<dyn Fn>` state plumbing), and the provided default
  makes it zero-cost to ignore. tonic precedent: one `Status` presentation owned by the service
  layer.
- **One hook, not two**: an earlier sketch had separate `render_rejection` / `render_internal`
  methods; folding 500s in as `RejectionKind::Internal` keeps one concept and one override point —
  an envelope that differs between 4xx and 5xx just matches on `kind`.
- **`#[non_exhaustive]` kind enum**: new surfaces (e.g. plan 09's multipart kinds) must be
  addable without breaking downstream implementations; the wildcard arm the attribute forces is
  the deliberate trade.
- **`Result`-wrapped extractors over rejection-in-extension tricks**: `IntoResponse` conversion
  has no access to state, so any design keeping bare extractor args cannot reach the
  implementation; `Result`-wrapping is axum's supported mechanism and keeps the funnel visible in
  generated code.
