# 07 — Server stub generation (tonic-style traits + an axum runtime)

Request (Roman, 2026-06-13): "progenitor generates a client only. Add a feature
flag that lets users generate a **server stub** as well, designed like tonic +
prost: generate a **trait** the user implements; once implemented they get a
full server that interoperates perfectly with the generated client. The trait
has one method per API route, takes a **typed request** and returns a **typed
response**, both wrapped (like tonic's `Request<T>`/`Response<T>`) so they can
also carry metadata — for REST that's headers, query params, etc. Implementors
must not touch serialization or header parsing. A **new optional runtime crate**
(axum-based) actually runs the server: build it with host + port + config,
register one or more services (the user types implementing the generated trait).
The runtime crate is optional and **must not be re-exported by the build.rs-side
generator crate**."

Status: **plan only — nothing implemented**. This document is meant to be
iterated on until an agent can implement it end to end. All progenitor/tonic
references are `file:line` against the tree at branch `feat/openapi3.1-support`
(progenitor) and a shallow clone of `hyperium/tonic` under
`target/plan-scratch/reference/tonic` (gitignored; safe to delete).

---

## 1. Goal, in one paragraph

Add an **opt-in** generation mode that, alongside the existing client, emits (a)
an `#[async_trait]` **service trait** with one `async fn` per **supported**
operation (unsupported ops get a 501 route stub, not a trait method — §6.5/D-skip)
taking a typed, wrapped request and returning a typed, wrapped response/error, and (b) a
generated **`{Api}Server<T>`** adapter that turns any `T: {Api}` into an
`axum::Router` — wiring every route, doing all deserialization of path/query/
header/body and all serialization of responses/errors, so the implementor writes
only business logic. A **new optional runtime crate `progenitor-server`** carries
the shared wrapper types (`Request<T>`, `Response<T>`, `ServerError`), the
extraction/response glue, and a tonic-style `Server::builder().add_service(..)
.serve(addr)` convenience. The server a user implements is wire-compatible with
the client progenitor generates from the same spec — that round-trip is the
headline acceptance test.

Non-goals for v1 (tracked in §12): websocket/upgrade endpoints, SSE/streaming
response bodies, multipart, per-tag trait splitting, OpenAPI-level auth
enforcement. These degrade gracefully — a **501 route stub** (no trait method) plus
a build warning, never a silent drop (§6.5, D-skip) — analogous to how httpmock
skips the synthesized multi-kind responses today
(`crates/progenitor-impl/src/httpmock.rs:379-384`).

---

## 2. What exists today (grounded)

Progenitor is **client-only**. The generation surface:

- `Generator::generate_tokens` (`crates/progenitor-impl/src/lib.rs:409`) emits
  the whole client file: a `pub mod types { … }` from typify, a `struct Client`,
  `impl ClientInfo`, `impl ClientHooks`, and the operation methods
  (`lib.rs:520-605`). `generate_text` (`lib.rs:373`) is the formatted wrapper
  (tokens → `syn::parse2` → prettyplease).
- Two **secondary, self-contained generators** already follow the pattern we
  want to copy: `Generator::httpmock(spec, crate_path)`
  (`crates/progenitor-impl/src/httpmock.rs:31`) and `Generator::cli(spec,
  crate_name)` (`crates/progenitor-impl/src/cli.rs:24`). Each is a separate
  `pub fn … -> Result<TokenStream>` entry point that re-walks
  `document.operations`, references the SDK's types through a caller-supplied
  crate path (`use #crate_path::*;`), and is invoked out-of-band (conformance
  build.rs, cargo-progenitor) rather than folded into `generate_tokens`. **The
  server generator is a third member of this family.**
- The per-operation IR is already everything we need
  (`crates/progenitor-impl/src/method.rs`):
  - `OperationMethod` (`method.rs:19`): `operation_id`, `tags`, `method`
    (`HttpMethod`), `path` (`PathTemplate`), `summary`/`description`, `params`,
    `responses`.
  - `OperationParameter` (`method.rs:100`) + `OperationParameterKind`
    (`method.rs:120`): `Path | Query(required) | Header(required) | Body(
    BodyContentType)`. `OperationParameterType` is `Type(TypeId) | RawBody`.
  - `BodyContentType` (`method.rs:144`): `Json | FormUrlencoded | OctetStream |
    Text(..) | Raw(..)`.
  - `OperationResponse` (`method.rs:255`) with `OperationResponseStatus`
    (`method.rs:288`: `Code(u16) | Range(u16) | Default`) and
    `OperationResponseKind` (`method.rs:346`: `Type(TypeId) | None | Raw |
    Upgrade | Synth(name)`).
  - `Generator::extract_responses(method, predicate)` partitions responses into
    success vs error and computes the unified success/error Rust types — used by
    the client at `method.rs:1284` (success) and `method.rs:1341` (error). **The
    server trait must reuse this so client and server agree on types.**
  - `PathTemplate` (`crates/progenitor-impl/src/template.rs:16`): `compile`
    (client `format!`), `names` (`template.rs:52`), `as_wildcard` /
    `as_wildcard_param` (`template.rs:63,75`, the httpmock regexes). We will add
    one more renderer: `as_axum_path`.
- The runtime split we mirror: `progenitor-client`
  (`crates/progenitor-client/src/progenitor_client.rs`) holds `ClientInfo`,
  `ResponseValue`, `Error`, `ByteStream`, encode/dispatch helpers. Generated
  client code references it as `progenitor_client::*` (`lib.rs:520-545`). For the
  standalone CLI output it can be **vendored inline** via
  `progenitor_client::code()` (`crates/progenitor-client/src/lib.rs:15`).
- Settings & flag plumbing:
  - `GenerationSettings` (`lib.rs:79`) with `InterfaceStyle`
    (`lib.rs:108`: `Positional|Builder`) and `TagStyle` (`lib.rs:123`:
    `Merged|Separate`), set via `with_interface`/`with_tag` (`lib.rs:143,149`).
  - The macro maps its input to settings at
    `crates/progenitor-macro/src/lib.rs:341-399`.
  - `cargo-progenitor` exposes flags via clap (`crates/cargo-progenitor/src/
    main.rs:26-88`) and writes the crate (`main.rs:122-196`); `--include-client`
    controls whether `progenitor_client.rs` is vendored.
  - Conformance: `conformance_support::generate("spec.toml")`
    (`conformance/support/src/lib.rs:50`) writes `$OUT_DIR/codegen.rs`,
    `example_tests.rs`, and `mock.rs` (`support/src/lib.rs:128-131`,
    `generate_mock_helpers` at `:199`). Workspace is separate (`Cargo.toml:17`
    `exclude = ["conformance"]`).
- Workspace deps already present (root `Cargo.toml`): `http = 1.4`,
  `hyper = 1.9`, `tokio = 1.52`, `serde`, `serde_json`, `serde_urlencoded`,
  `bytes`, `futures`, `thiserror`, `percent-encoding`. **`axum` is not yet a
  dependency** — we add it (0.8.x, which is on http 1.x / hyper 1.x). MSRV is
  `1.88` (root `Cargo.toml:21`), so native `async fn` in traits is available;
  see the async-trait decision in §12.

---

## 3. Reference study: how tonic does it (and the REST mapping)

Distilled from the clone. tonic-build's **server** codegen
(`tonic-build/src/server.rs`) emits, per proto `service`:

1. A trait (`generate_trait` → `generate_trait_methods`, `server.rs:234-357`),
   annotated `#[async_trait]` (`server.rs:227`), one method per RPC:
   ```rust
   async fn #name(&self, request: tonic::Request<#Req>)
       -> std::result::Result<tonic::Response<#Res>, tonic::Status>;   // server.rs:281-283
   ```
2. A `#ServiceServer<T>` struct holding `inner: Arc<T>` plus config
   (`server.rs:116-149`), with `new(inner)` / `from_arc(Arc<T>)`.
3. `impl<T: Trait, B> tower::Service<http::Request<B>> for #ServiceServer<T>`
   that **routes on `req.uri().path()`** to per-method code, returning
   `Unimplemented` for unknown paths (`server.rs:151-178`).
4. Per method a `#MethodSvc<T>(Arc<T>)` implementing `UnaryService`, whose
   `call` invokes `<T as Trait>::#method(&inner, request)` (`server.rs:459-495`);
   a codec (prost) does (de)serialization.

The runtime (`tonic/src/…`):
- `Request<T>` = `{ message: T, metadata: MetadataMap, extensions: Extensions }`
  (`request.rs:16-22`); `Response<T>` mirrors it (`response.rs:5-11`).
- `Status` = universal error: code + message + details + metadata + source
  (`status.rs:38-60`).
- `Server::builder()` → `.add_service(svc) -> Router` (`transport/server/mod.rs:
  511`) → `Router::add_service` chains more (`:979`) → `.serve(addr).await`
  (`:1021`). Internally the router is **axum** (`add_service` bound:
  `S::Response: axum::response::IntoResponse`, `:519`).
- `async_trait` is re-exported through `tonic::codegen` and the generated trait
  relies on it being in scope (`tonic/Cargo.toml:125`, `use tonic::codegen::*`).

### 3.1 The gRPC → REST impedance mismatch and our resolution

| tonic (gRPC) | progenitor (REST) — this plan |
|---|---|
| One proto `service` → one trait | Whole API → **one trait** (v1; per-tag later, gated on existing `TagStyle`) |
| Method has exactly **one** message | Operation splits inputs across **path + query + header + body** → bundle into one generated **per-operation request struct** = the `T` in `Request<T>` |
| `Request<T>` carries gRPC headers as metadata | `Request<T>` carries **untyped extras**: full `http::HeaderMap`, `Extensions`, `ConnectInfo` (typed/declared params live in `T`, not metadata) |
| `Response<T>` + `Status` | `Response<T>` (typed body + status + headers) + typed per-op error (see §6.4) |
| prost codec | serde_json (+ raw bytes / urlencoded / text) |
| Hand-rolled `tower::Service` routing on URI | **axum `Router`** does routing + extraction; `{Api}Server<T>` produces the `Router` |
| `Server::builder().add_service().serve()` | **Same shape**, thin wrapper over axum + tokio |

The one deliberate divergence from the request's wording: the user lumped "query
params" into metadata. We instead put **all spec-declared, typed params (path,
query, header) into the typed message `T`**, and reserve the generic metadata
wrapper for *undeclared* extras. Rationale: the stated #1 requirement is
"implementors must not worry about serialization / header parsing." Putting
declared params in `T` is exactly what removes that boilerplate; leaving them as
stringly metadata would reintroduce it. (If we later want raw access to a
declared header too, it is still reachable via `request.headers()`.)

---

## 4. Design overview — three pieces

1. **Generator** (`crates/progenitor-impl`): a new self-contained entry point
   `Generator::server(spec, crate_path) -> Result<TokenStream>` (sibling of
   `httpmock`/`cli`), plus a `GenerationSettings` toggle so the macro/conformance
   paths can fold it into the primary output. It emits the **body** of a `server`
   module (the trait, per-op request/query structs, per-op error types, the
   responders, and the `{Api}Server<T>` → `axum::Router` adapter) — the consumer
   wraps it in `pub mod server { … }` exactly once (§7.1, Finding 4). Emitted
   **only** when enabled; zero axum/server footprint otherwise.
2. **Generated code shape** (§6): trait + wrappers + router wiring, referencing
   the runtime as `progenitor_server::*` (never inlined; see §9).
3. **Runtime crate `progenitor-server`** (§6.6): `Request<T>`, `Response<T>`,
   `ServerError`, extractor/response glue, `codegen` re-export module
   (`async_trait`, `axum`, `http`, …), and the `Server`/`Router` convenience.

Flag plumbing (§7) threads one switch through `GenerationSettings` →
`progenitor-macro` (`server` config key + cargo feature) → `cargo-progenitor`
(`--server`; no `--no-client` in v1, §7.3/D-server-only) →
`conformance_support::generate` (emit `$OUT_DIR/server.rs`).

---

## 5. Worked example (petstore-31) — the target output

Given `GET /pets?limit`, `POST /pets` (body `Pet`, 201 → `Pet`), `GET
/pets/{petId}` (200 → `Pet`, default → `Error`), the server module should look
like this (abbreviated; `types::*` are the **same** types the client uses). The
generator emits the module *body*; the `pub mod server { … }` wrapper shown here
is added by the consumer (§7.1, Finding 4):

```rust
pub mod server {
    use crate::types;                  // = #crate_path::types; "crate" is the default (§6.1)
    use progenitor_server::codegen::*; // async_trait, axum, http, Arc, …

    // 5.1 One bundled request struct per operation (the `T` in Request<T>).
    #[derive(Debug, Clone)]
    pub struct ListPetsRequest { pub limit: Option<i64> }
    #[derive(Debug, Clone)]
    pub struct CreatePetsRequest { pub body: types::Pet }
    #[derive(Debug, Clone)]
    pub struct ShowPetByIdRequest { pub pet_id: String }

    // 5.2 Per-operation typed error (reuses the client's error type; see §6.4).
    //     `ServerError<E>` adds the infrastructure/500 escape hatch.
    pub type ListPetsError      = progenitor_server::ServerError<types::Error>;
    pub type CreatePetsError    = progenitor_server::ServerError<types::Error>;
    pub type ShowPetByIdError   = progenitor_server::ServerError<types::Error>;

    // 5.3 The trait the user implements. One async fn per route.
    #[async_trait]
    pub trait Petstore: Send + Sync + 'static {
        async fn list_pets(
            &self,
            request: progenitor_server::Request<ListPetsRequest>,
        ) -> Result<progenitor_server::Response<Vec<types::Pet>>, ListPetsError>;

        async fn create_pets(
            &self,
            request: progenitor_server::Request<CreatePetsRequest>,
        ) -> Result<progenitor_server::Response<types::Pet>, CreatePetsError>;

        async fn show_pet_by_id(
            &self,
            request: progenitor_server::Request<ShowPetByIdRequest>,
        ) -> Result<progenitor_server::Response<types::Pet>, ShowPetByIdError>;
    }

    // 5.4 The adapter: any `T: Petstore` → an axum::Router. Hides all (de)ser.
    pub struct PetstoreServer<T>(Arc<T>);
    impl<T: Petstore> PetstoreServer<T> {
        pub fn new(inner: T) -> Self { Self(Arc::new(inner)) }
        pub fn from_arc(inner: Arc<T>) -> Self { Self(inner) }
        pub fn into_router(self) -> axum::Router {
            axum::Router::new()
                .route("/pets",        axum::routing::get(Self::list_pets_route)
                                          .post(Self::create_pets_route))
                .route("/pets/{petId}", axum::routing::get(Self::show_pet_by_id_route))
                .with_state(self.0)
        }
        // one extractor->call->responder per op; all args are custom
        // progenitor_server::* extractors (Metadata + Query/Json/Path), §6.5
        async fn list_pets_route(/* State, Metadata, progenitor_server::Query<…> */) -> axum::response::Response { /* … */ }
        async fn create_pets_route(/* State, Metadata, progenitor_server::Json<Pet> */) -> axum::response::Response { /* … */ }
        async fn show_pet_by_id_route(/* State, Metadata, progenitor_server::Path<String> */) -> axum::response::Response { /* … */ }
    }

    // 5.5 tonic-style ergonomics: NamedService so the runtime can mount it.
    impl<T: Petstore> progenitor_server::Service for PetstoreServer<T> {
        fn into_router(self) -> axum::Router { PetstoreServer::into_router(self) }
    }
}
```

User code — note how close this is to tonic:

```rust
struct MyPets;                                  // their state
#[async_trait]
impl server::Petstore for MyPets {
    async fn show_pet_by_id(&self, req: Request<server::ShowPetByIdRequest>)
        -> Result<Response<types::Pet>, server::ShowPetByIdError>
    {
        let id = req.get_ref().pet_id.clone();   // typed, already parsed
        let auth = req.headers().get("authorization");   // raw metadata if needed
        Ok(Response::new(types::Pet { id: 1, name: "Fido".into(), tag: None }))
    }
    // … list_pets, create_pets …
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    progenitor_server::Server::builder()
        .add_service(server::PetstoreServer::new(MyPets))
        .serve("0.0.0.0:8080".parse()?)
        .await?;
    Ok(())
}
```

Power-user path (no runtime lock-in; compose with their own axum app):
```rust
let app = server::PetstoreServer::new(MyPets).into_router()
    .layer(tower_http::trace::TraceLayer::new_for_http());
axum::serve(listener, app).await?;
```

---

## 6. Detailed design

### 6.1 Trait granularity and naming

- v1: **one trait per API**, named from `info.title` sanitized to PascalCase
  (e.g. `Petstore`), in the generated `server` module (the consumer-wrapped body,
  §7.1). Adapter `{Trait}Server<T>`. This matches the default `TagStyle::Merged`
  and the single-`Client` client today.
- Method name = `operation_id` after the same dedup that
  `generate_tokens` applies (`lib.rs:432-447`), so trait methods line up 1:1
  with client methods.
- Trait bound `Send + Sync + 'static` (axum handlers are spawned and shared).
- **`crate_path` semantics (Finding 1, mirrors `httpmock`).** The body references
  the spec types through `use #crate_path::types;` (and `progenitor_server::codegen::*`
  for the runtime), where `crate_path` is `"crate"` for the common in-crate case
  (macro / build.rs / cargo-progenitor all emit client + types + server into one
  crate) — rendering to the absolute `use crate::types;`, which resolves no matter
  how deep the `server` module is wrapped. An external `crate_path` (e.g. a
  separate server crate referencing a published types crate) renders
  `use that_crate::types;`. One API handles both — no separate "in-crate" vs
  "out-of-crate" entry point (D-crate-path). **Never** `use super::types` (breaks
  under deeper nesting).
- **Follow-up (gated, not v1):** when `TagStyle::Separate`, emit one trait per
  tag (`PetstorePetsService`, …) and let the adapter merge their routers, just as
  the client does `generate_tokens_builder_separate` (`lib.rs:465`). Keep the
  single-trait and per-tag outputs behind the same setting the client already
  uses so the two stay consistent.

### 6.2 The wrapper types (`progenitor-server` runtime)

Direct analogs of tonic's `request.rs`/`response.rs`, REST-flavored:

```rust
pub struct Request<T> {
    message: T,
    meta: Metadata,   // method, uri, headers, extensions — the §6.5 extractor's type
}
impl<T> Request<T> {
    pub fn new(message: T) -> Self;                 // empty Metadata, for tests/fixtures
    pub fn from_metadata(meta: Metadata, message: T) -> Self;
    pub fn get_ref(&self) -> &T;
    pub fn get_mut(&mut self) -> &mut T;
    pub fn into_inner(self) -> T;
    pub fn headers(&self) -> &http::HeaderMap;
    pub fn extensions(&self) -> &http::Extensions;  // ConnectInfo, auth results
    pub fn method(&self) -> &http::Method;
    pub fn uri(&self) -> &http::Uri;
    pub fn into_parts(self) -> (Metadata, T);
}

pub struct Response<T> {
    message: T,
    status: Option<http::StatusCode>,   // None → spec's default success status
    headers: http::HeaderMap,
}
impl<T> Response<T> {
    pub fn new(message: T) -> Self;
    pub fn with_status(self, code: http::StatusCode) -> Self;
    pub fn headers_mut(&mut self) -> &mut http::HeaderMap;
    pub fn into_inner(self) -> T;
}
```

`Request<T>` carries a `Metadata` (the same type the axum extractor produces,
§6.5) rather than a bespoke `MetadataMap` — REST headers are just
`http::HeaderMap`, so there's no need to reinvent tonic's ASCII/bin metadata
split, and holding cloned individual parts sidesteps the un-constructible
`http::request::Parts` problem (Finding 3). This is the "generic wrapper carrying
metadata" the request asked for, with declared params already pulled into the
typed `message`.

### 6.3 The per-operation request struct (the typed message `T`)

Generated per operation by walking `method.params`
(`OperationParameterKind`, `method.rs:120`):

- All fields use the param's **owned** type — typify `ident()` (e.g.
  `String`/`i64`/a newtype), **not** the client's borrowed
  `parameter_ident_with_lifetime` (`method.rs:903`): the struct is moved into the
  trait method and outlives extraction, so it can't hold `&str`/`&T` (Finding 6).
- `Path` → required owned field (`String` when `RawBody`/opaque).
- `Query(required)` → field on a generated `{Op}Query` helper; `Option<T>` when
  `!required`.
- `Header(required)` → field; `Option<T>` when `!required`.
- `Body(content_type)`:
  - `Json` / `FormUrlencoded` with `Type(id)` → `pub body: <owned typed>`.
  - `OctetStream` / `Raw` → `pub body: bytes::Bytes` (D-raw: bytes, not a stream —
    streaming is out of scope for v1).
  - `Text` → `pub body: String`.
- Field names reuse the param's sanitized `name`; the wire name (`api_name`) is
  what extraction binds to (mirrors the client/httpmock, `httpmock.rs:146-153`).
- Derives: the **request bundle** needs only `Debug, Clone` — it is assembled from
  per-source extractors, never deserialized whole. The generated **`{Op}Query`
  helper**, however, is *server-authored*, so the generator must emit
  `#[derive(::serde::Deserialize)]` on it explicitly — typify only derives the
  types *it* generates, not this one (Finding 6). The body type's derives come
  from typify. (Query *parsing* uses the custom extractor of §6.5, not axum's
  built-in `Query`.)

Design alternative (positional args, mirroring the client method signature) is in
§12 D-shape; recommendation is the bundle (Design A) for fidelity to the
"generic wrapper" requirement and forward-compatibility.

### 6.4 Success/error typing and the response-encoding contract

Reuse the existing partition logic, but the server must **own the encoding** —
this is where a "typed server" and a "wire-compatible client" can silently
diverge, so it is specified explicitly (review Findings 1 & 2).

**Deriving the response set.** Only after the shared preparation step (§6.9) has
populated `schema_supertypes`/`schema_type_ids` — without which
`extract_responses` panics, it reads them at `method.rs:1719-1720` (Finding 4) —
call:
- `extract_responses(method, is_success_or_default)` → `(success_items,
  success_kind)` (the client uses this at `method.rs:1284`),
- `extract_responses(method, is_error_or_default)` → `(error_items, error_kind)`
  (`method.rs:1341`).

Use the **items** (each an `OperationResponse` = status + `OperationResponseKind`)
to drive a generated responder; use the **kind** only to *name* the `Response<T>`
payload / error payload type.

**Server-side response-kind mapping — do NOT reuse the client's `into_tokens`.**
`OperationResponseKind::into_tokens` (`method.rs:362`) and `synth_enum_definition`
(`method.rs:1606-1620`) emit **client** types: `Raw → ByteStream`, `Upgrade →
::reqwest::Upgraded`, and a status-tagged `Synth` enum that *embeds those*. The
server maps instead:

| kind | `Response<T>` payload | wire encoding done by the generated responder |
|---|---|---|
| `Type(X)` | `X` | `serde_json` body + `content-type: application/json` |
| `None` | `()` | empty body, status only |
| `Raw` | `bytes::Bytes` | passthrough; content-type per §6.8 + the IR extension (D-raw: bytes only, no stream) |
| `Upgrade` | — | **skipped** (§6.8); op gets a 501 route stub, no trait method |
| `Synth(_)` | a **server-generated** per-op enum (its own, not the client's) | `match` the active variant → emit that variant's status + body via the rows above |

So a synth response is **never** JSON-serialized as an enum (the original plan's
mistake): the responder matches the active variant and emits the original payload
with that variant's status. The server's synth enum is generated server-side (no
`ByteStream`/`reqwest::Upgraded`; `Raw` variants carry `bytes::Bytes`), so the
client and server synth enums are *parallel but distinct* — both derived from the
same `(status, kind)` item list, which is what guarantees wire agreement without
sharing a type.

**Default success status:** lowest concrete 2xx in `success_items`, else 200;
emitted as a generated const, applied when `Response.status` is `None`.

**Error model — D-error DECIDED (2026-06-13, per review): hybrid, with
per-operation generated encoding.** `ServerError<E>` is a **data** type only:
```rust
pub enum ServerError<E> {
    Api { status: http::StatusCode, body: E },  // a declared, typed error
    Internal(BoxError),                          // → 500
    Response(http::Response<axum::body::Body>),  // fully custom escape hatch
}
impl<E> ServerError<E> {
    pub fn api(status: http::StatusCode, body: E) -> Self;
    pub fn internal(e: impl Into<BoxError>) -> Self;
}
```
Crucially there is **no** blanket `impl<E: Serialize> IntoResponse for
ServerError<E>` (Finding 2): a generic impl cannot know whether `E = ()` means an
empty body, whether `E` is raw vs JSON, or whether an `Api{ status, body }` whose
`body` is a synth variant carries a status matching that variant. Instead each
operation gets a **generated responder** — `fn {op}_respond(Result<Response<Succ>,
{Op}Error>) -> axum::response::Response` — that encodes *both* arms with the table
above for that operation's concrete kinds. The `Internal`/`Response` arms have
fixed encodings the runtime supplies (500 / passthrough); only the typed `Api`
arm needs per-op knowledge. The per-op alias `type ShowPetByIdError =
ServerError<types::Error>` (§5.2) still names the data type.

**Status resolution contract (Finding 5).** The responder derives the HTTP status
from the response *value*, never from a separate override that could disagree:
- **Success `Response<T>` for a plain / `None` / `Raw` `T`:** status =
  `Response.with_status` override if set, else the generated default-success const
  (lowest 2xx). An override is **validated against the operation's declared success
  set** (a declared 2xx code, an in-range success `Range`, or a success `Default`);
  an off-contract override is a programming error → **500** + log (Finding 2). For
  a deliberately off-contract success, use the `ServerError::Response` escape hatch.
- **Synth success/error enums:** the *server-side* enum carries the status where
  it isn't implied — a `Code(n)` variant **implies** status `n`; `Range(r)` and
  `Default` variants carry an explicit `status: http::StatusCode` field (there is
  no concrete code to imply). The responder reads status from the active variant
  and, for `Range(r)`, validates `status / 100 == r`.
- **Error `ServerError<E>` with a plain `E`:** `Api { status, body }` emits
  `status` + the body **encoded per the response-kind table** — typed `E` → JSON,
  `E = ()` → empty body, `E = bytes::Bytes` → raw passthrough (error responses
  aren't always JSON, Finding 2). The implementor picks `status` (typically the
  declared error status). `Internal` → 500; `Response(r)` → `r` verbatim (hatch).
- **Conflict / out-of-range:** any contradiction the responder can detect — an
  override disagreeing with a concrete synth variant, a `Range` status out of
  band, a `Range`/`Default` variant with no status — is a **server-side
  programming error**: the responder returns **500** and logs, never emitting an
  off-contract status the client wouldn't match.

**Status-validation matrix** (the authoritative lookup; rejection → 500 + log,
escape hatch = `ServerError::Response`):

| response shape | status written | validated against | on violation |
|---|---|---|---|
| plain success, no override | default-success const (lowest 2xx) | — | — |
| plain success, `with_status` override | the override | declared success codes/ranges/`Default` | 500 + log |
| `None` success | as plain success | as plain success | 500 + log |
| `Raw` success | as plain success; body passthrough, content-type §6.8 | as plain success | 500 + log |
| synth success — `Code(n)` variant | `n` | `with_status` (if any) must equal `n` | 500 + log |
| synth success — `Range(r)`/`Default` variant | variant's carried `status` | `Range(r)`: `status/100 == r`; `with_status` (if any) must equal the carried status | 500 + log |
| plain error `Api{status,body}` | `status`; body per response-kind table (typed→JSON, ()→empty, Bytes→raw) | client error-arm match (note) | 500 + log |
| synth error variant | variant status (as synth success rows) | as synth success + error-arm match | 500 + log |
| `ServerError::Internal` | 500 | — | — |
| `ServerError::Response(r)` | `r` verbatim | — | — (deliberate escape hatch) |

For **all** synth variants (Finding 3) the variant is the *sole* status source;
`Response::with_status` is accepted only when it equals the implied (`Code(n)`) or
carried (`Range`/`Default`) status, else 500 + log. S0.5 covers `with_status` on
Code, Range, and Default synth variants.

**Status validator = a shared, mechanical classifier (Findings 2-3).** Do **not**
model this as "success then error then Default": in the real client `Default`/
`Range` are *embedded* into the arm patterns — non-synth success collapses
`Range`/`Default` to `200..=299` (`success_arm_pattern`, `method.rs:411-431`),
synth `Default` → `_`, and a final `default_response` arm is emitted only when a
side declares a default (`method.rs:1456`). So define **one shared classifier**
that emits/evaluates the *same* ordered arm patterns the client generates from the
response items and classifies a candidate status by first match →
`Success(kind)` | `Error(kind)` | `Unexpected`. Both the client's arm generation
and the server validator call it, so they cannot drift. Validation rule: a success
responder's status must classify as `Success`; an error responder's status must
classify as `Error` (so an error `Default` cannot accept a status a success arm
claims first — a success `200..=299` would make the client decode a 200 "error
default" as success). Otherwise 500 + log. This classifier lives in §6.9's shared
prep.

**Ergonomic constructors (Finding 3).** To avoid duplicated status authority
between `Api { status, body }` and a synth variant that already implies/carries
its status, the generator emits, per synth error enum, a `From<{SynthError}> for
ServerError<{SynthError}>` (and per-variant helpers) that builds `Api { status,
body }` with the status **derived from the variant** — concrete for `Code(n)`,
carried for `Range`/`Default`. The common path is then `return
Err(MyError::Status404(body).into())`, which can't mismatch. Explicit
`ServerError::api(status, body)` stays for plain-type errors and as a validated
override.

This keeps the wire status inside the set the generated client matches on
(`method.rs:1352-1361`), which is what preserves interop.

`Response<T>` and the success/error halves flow through that generated per-op
responder (next section), not a generic `IntoResponse`.

### 6.5 The adapter `{Api}Server<T>` → `axum::Router` (boilerplate lives here)

Per operation, generate one `async fn {op}_route` axum handler that centralizes
all extraction and response encoding so the implementor never touches it.

**Extractor composition (Finding 3 — prototype this in S0 before writing the
generator).** axum runs `FromRequestParts` extractors in argument order plus **one**
final body-consuming `FromRequest` extractor. You cannot *also* take the whole
`http::request::Parts` by value (there is no `FromRequestParts for Parts`, and the
body can't be moved out twice) — so the original `parts: http::request::Parts`
handler arg would not compile. The metadata wrapper is instead supplied by a
runtime `Metadata` extractor that implements `FromRequestParts` and clones the
method/uri/headers/extensions *before* the body extractor runs:

```rust
// in progenitor-server
pub struct Metadata { /* method, uri, headers, extensions */ }
impl<S: Sync> axum::extract::FromRequestParts<S> for Metadata { /* clone the bits */ }
```

A generated handler then composes cleanly (body extractor last):

```rust
async fn create_pets_route(
    State(inner): State<Arc<T>>,        // FromRequestParts
    meta: Metadata,                     // FromRequestParts (our extractor); also the header source
    // Path(..) / Query(..) here — custom progenitor_server extractors
    body: progenitor_server::Json<types::Pet>,  // custom body extractor, LAST
    // typed headers are parsed in-body from `meta` (see Headers rule below)
) -> axum::response::Response {
    let message = CreatePetsRequest { body: body.0 };
    let request = progenitor_server::Request::from_metadata(meta, message);
    let result = <T as Petstore>::create_pets(&inner, request).await;
    create_pets_respond(result)         // generated per-op responder (§6.4)
}
```

**Rejections are runtime-standard, not per-op typed (Finding 1, decided).** Bare
axum extractors emit *their own* `IntoResponse` rejection (e.g. a plain-text 422)
and short-circuit **before** the handler, so the custom `progenitor_server`
extractors (`Json`/`Form`/`Bytes`/`Text`, `Path`, `Query`) replace them. But an
extractor's `Rejection` still short-circuits before `{op}_respond`, and a generic
`progenitor_server::Json<T>` cannot know operation X's declared error *type*. So
the decision: **extraction failures are runtime-standard errors** — a uniform
`progenitor_server` body with the right status (`400` malformed / `415` wrong
content-type), **not** the operation's typed error. (The earlier "shaped like a
declared error where possible" overstated this and is removed.) Consequence (same
caveat as the 501 stub, D-501): the generated *client* may surface such a 400 as
`InvalidResponsePayload` (`.status()` `None`); the true status is observable via
raw HTTP, and tests assert it that way. **Typed extraction errors are opt-in /
future:** for an op that declares a fillable 4xx validation error, take
`Result<Extractor, Rejection>` args and map the rejection into that error through
`{op}_respond` — but v1 ships the runtime-standard path. Clean handler signatures
(no `Result<_, _>` per arg) are kept for the v1 path.

Ops with **no** body use only `FromRequestParts` extractors (no final body
extractor) — also fine.

Extraction rules (all via custom `progenitor_server` extractors, per the
rejection note above):
- Path: `progenitor_server::Path<T>` (or a tuple) in path order; a type-parse
  failure rejects with the runtime's `400`.
- Query: a generated `#[derive(::serde::Deserialize)] struct {Op}Query { … }`
  extracted via a **custom `progenitor_server::Query` extractor — NOT
  `axum::extract::Query`** (Finding 1). The client serializes array params as
  repeated keys (`?foo=1&foo=2`, via `QuerySeq`,
  `progenitor_client.rs:789,845-867`); axum's built-in `Query` uses plain
  `serde_urlencoded`, whose docs say it does *not* support repeated keys
  (`axum-0.8/src/extract/query.rs:45`). The runtime extractor uses a repeated-key
  deserializer (e.g. `serde_html_form`) so it round-trips what the client emits.
  Optional fields `Option<T>`. deepObject/exploded styles the client special-cases
  (httpmock skips them, `httpmock.rs:140-144`) get skip-with-`#[allow]` + `// TODO
  deepObject` in v1; raw query stays reachable via `meta.uri().query()`.
- Headers: parsed **inside the generated route** from `meta.headers()` via two
  runtime helpers — `progenitor_server::required_header::<T>(&meta, api_name) ->
  Result<T, Rejection>` and `optional_header::<T>(&meta, api_name) ->
  Result<Option<T>, Rejection>` (review 6 Finding 3). A name-less runtime
  typed-header extractor can't know the per-op header name/type without generated
  marker types; parsing in the route, where both are known, is simpler. Because
  these helpers run *in-body* (unlike extractor args, which short-circuit on their
  own), the generated route **early-returns** their `Rejection` as the
  runtime-standard response **before** constructing the request message:
  ```rust
  // generated, inside {op}_route, before building {Op}Request:
  let x_request_id = match progenitor_server::optional_header::<Uuid>(&meta, "x-request-id") {
      Ok(v)   => v,
      Err(rej) => return rej.into_response(),   // runtime-standard 400, trait not called
  };
  // required headers use required_header and bind T directly (same early-return).
  ```
  A missing-required / unparseable header → `400` (runtime-standard), never
  reaching the trait.
- Body by `BodyContentType`: custom `progenitor_server::Json<T>` / `Form<T>` /
  `Bytes` / `Text` (json / urlencoded / octet-stream+raw / text — `Bytes`/`Text`
  are non-generic, review 6 Finding 4) — *not* the bare axum extractors. A rejected
  body (malformed JSON / wrong content-type) → `400`/`415`
  from the extractor's own `Rejection` (the runtime's standardized response),
  before the trait.
- The `Result` is handed to the generated `{op}_respond` (§6.4), which owns status
  + serialization. There is no generic `into_axum_ok`/`IntoResponse` shortcut.

Routing: group operations by **path template**, register methods on each
`axum::routing::MethodRouter` (`.get(..).post(..)`), `.with_state(Arc<T>)`.
Skipped/unsupported ops (upgrade, multipart) still get a route returning **501
Not Implemented** so the surface stays complete (Finding 9, §12 D-skip). Caveat
(Finding 2): the *generated client* may not surface that 501 cleanly — for an op
with a typed/`Default`/range error the client decodes the error body
(`method.rs:1360-1383`), and an empty/plain-text 501 becomes
`InvalidResponsePayload`, whose `.status()` is `None`
(`progenitor_client.rs:167,372`). So the 501 stub serves *direct* HTTP callers;
tests assert it via raw HTTP and the generator emits a build **warning** listing
skipped ops (§10, §12 D-501). Path translation in §6.7.

### 6.6 The runtime crate `progenitor-server`

New `crates/progenitor-server` (workspace member; runtime-only; **not** a build
dep). Modules:

- `request` / `response` / `error`: the types in §6.2 / §6.4.
- `extract`: the `Metadata` extractor (§6.5); the custom request extractors with
  **runtime-standard** rejections — `Path`, the repeated-key `Query` (backed by
  `serde_html_form`), and the body extractors `Json`/`Form`/`Bytes`/`Text`
  (Finding 1); the `required_header::<T>()` / `optional_header::<T>()` parse
  helpers used in-route (Finding 4 — headers are *not* a name-less extractor); plus the
  rejection→standardized-response conversions.
- `service`: **always available** (not behind `transport`) — the `Service` trait
  every generated `{Api}Server` implements (Finding 5):
  ```rust
  pub trait Service { fn into_router(self) -> axum::Router; }
  ```
  It needs only `axum` (non-optional), so power users can `into_router()` and
  bring their own server without the `transport` feature.
- `codegen`: a `pub use` surface the generated code imports via
  `use progenitor_server::codegen::*;` — re-exports `async_trait::async_trait`,
  `axum`, `http`, `std::sync::Arc`, `bytes::Bytes`, serde traits. (Exactly tonic's
  `tonic::codegen::*` trick, `tonic/Cargo.toml:125`.) This keeps generated code
  terse and lets us swap implementation details without regenerating consumers.
- `transport` (behind the `transport` feature — the only part needing `tokio` +
  serving; `Service` above is **not** gated): the convenience server.
  ```rust
  pub struct Server { /* layers, timeouts, body limit, … */ }
  impl Server {
      pub fn builder() -> Self;
      pub fn concurrency_limit(self, n: usize) -> Self;
      pub fn timeout(self, d: Duration) -> Self;
      pub fn layer<L>(self, layer: L) -> Self;          // tower layer
      pub fn add_service<S: Service>(self, svc: S) -> Router;
  }
  pub struct Router { /* merged axum::Router + Server config */ }
  impl Router {
      pub fn add_service<S: Service>(self, svc: S) -> Self;     // merge another
      pub fn into_make_service(self) -> /* axum */;
      // Bind first so callers/tests can read the ephemeral port (Finding 8):
      pub async fn bind(self, addr: SocketAddr) -> Result<Bound, Error>;
      pub async fn serve(self, addr: SocketAddr) -> Result<(), Error>;      // bind + run
      pub async fn serve_listener(self, l: tokio::net::TcpListener) -> Result<(), Error>;
  }
  pub struct Bound { /* TcpListener + router */ }
  impl Bound {
      pub fn local_addr(&self) -> SocketAddr;             // the actually-bound port
      pub async fn serve(self) -> Result<(), Error>;
      pub async fn serve_with_shutdown<F: Future<Output=()>>(self, signal) -> Result<(), Error>;
  }
  ```
  `add_service` merges `svc.into_router()` (`Router::merge`), applies the
  configured tower layers, and serving runs `axum::serve`. The `bind → Bound →
  local_addr → serve` path (Finding 8) is exactly what the round-trip test uses to
  grab an ephemeral `127.0.0.1:0` port; `serve_listener` lets callers pass their
  own listener. tonic shape (`transport/server/mod.rs:511,979,1021`), adapted to
  axum; multiple services = multiple specs / per-tag traits on one socket.

**Feature split (Finding 5).** `axum` is **non-optional** — every generated server
(the `{Api}Server` adapter, `into_router`, the `Metadata`/body extractors) needs
it, so a no-axum "core" tier would serve nobody who generates a server. Features:
- `default = ["transport"]`.
- `transport` gates **only** the tokio serving convenience (`Server`/`Router`/
  `Bound`/`serve*`): pulls `tokio` (rt-multi-thread, net) + `tower`/`tower-http`.
- With `transport` off you still get the wrappers, `ServerError`, the
  generated-code support (`codegen`, extractors), and `Service::into_router` → so
  you compose the `axum::Router` into your own server. (This replaces the earlier,
  self-contradictory "off = wrappers + into_router only", which couldn't hold
  because `into_router` needs axum.)

Cargo deps: `axum = "0.8"` (non-optional), `http`, `bytes`, `serde`,
`serde_json`, `serde_urlencoded`, `serde_html_form` (repeated-key query, Finding
1), `async-trait`, `futures`, `thiserror`; `tokio` (rt-multi-thread, net),
`tower`, `tower-http` behind `transport`. Add `axum`, `async-trait`,
`serde_html_form` (and `tower`, `tower-http`) to the **root**
`[workspace.dependencies]`.

### 6.7 Path template translation (OpenAPI → axum)

OpenAPI uses `{petId}`; **axum 0.8 also uses `{petId}`** (and `{*rest}` for
catch-all) — so translation is nearly identity. Add to `PathTemplate`
(`template.rs`):
```rust
pub fn as_axum_path(&self) -> String   // "/pets/{petId}" from the components
```
emitting `Constant` verbatim and `Parameter(p)` as `{p}` using the wire name. Add
a unit test next to the existing template tests. (We deliberately target axum 0.8
to avoid the 0.7 `:param` rewrite.)

### 6.8 Content types & response kinds beyond JSON

- Request bodies: JSON, urlencoded, octet-stream/raw (`Bytes`), text (`String`)
  per §6.5. Multipart → v1 skips the op (501 route stub, §6.5; same posture as the
  `Raw(..)` fallback the client uses, `method.rs:148-156`).
- Response kinds are encoded by the generated per-op responder per the §6.4 table
  — `Type`→JSON, `None`→empty, `Raw`→passthrough, `Synth`→match-by-variant,
  `Upgrade`→501 stub. It is **not** a generic serializer and **never**
  JSON-of-the-synth-enum.
- **Response content-type retention (Finding 7 — IR extension required).**
  `OperationResponse` (`method.rs:256-268`) stores `status_code`, `typ`,
  `schema_name`, `description` — **no media type**, so a `Raw` response's
  content-type is unknown today. Before promising raw content-type fidelity, add
  `media_type: Option<String>` to `OperationResponse`, populated in
  `process_operation` from **the exact content entry that produced the
  `Raw`/`OctetStream` classification** — the same selection that set the kind, not
  an independent re-scan (Finding 4), so kind and media type can't disagree. If a
  response declares several non-JSON media types with no single selection, v1 picks
  the first by media-type string sort (deterministic, documented), or skips the op
  (501) if even that is ambiguous; multi-media raw is rare. Until `media_type` is
  populated the responder emits `application/octet-stream` for `Raw` and
  `application/json` for `Type`, with a `// TODO content-type` where the IR is
  silent. This IR change belongs to S0/S2, not S4.
- Skipped/unsupported ops are never silently dropped: each gets a `// progenitor:
  skipped <op> (<reason>)` comment **and** a 501 route stub (§6.5), so routing
  stays complete and the gap is visible.

### 6.9 Shared "prepare generation IR" step (Finding 4)

`Generator::server` must **not** blindly mirror `httpmock` (which re-calls
`add_ref_types` itself, `httpmock.rs:34-39`, and never builds the supertype maps
or dedups operation IDs). Three hazards:
- `extract_responses` reads `self.schema_supertypes`/`self.schema_type_ids`
  (`method.rs:1719-1720`), populated **only** in `generate_tokens`
  (`lib.rs:419-421`). Calling it without that setup panics.
- Operation IDs are deduped in `generate_tokens` (`lib.rs:432-447`); the server's
  method/route names must match the client's exactly, or a generated server won't
  line up with its client.
- `add_ref_types` run twice over the same schemas risks double-adding types.

Fix: factor a single idempotent `prepare(&mut self, spec) -> PreparedIr` on
`Generator` that returns an **owned** value — **not** `&PreparedIr` (Finding 3). A
borrowed return is a trap: token generation needs `&mut self` (`lib.rs:659`) and
mutates flags like `uses_futures` (`method.rs:973`), so an outstanding immutable
borrow of `self` would block every generation call. `prepare` applies
`add_ref_types`, populates the `self.schema_*` maps (as `generate_tokens` already
does, `lib.rs:419-421`), and returns owned data callers pass by reference:
`PreparedIr { raw_methods: Vec<OperationMethod> /* deduped IDs */, … }` (and,
after Finding 7, each response's `media_type`). `generate_tokens`, `server`,
`httpmock`, and `cli` all consume `PreparedIr` instead of each re-deriving a
subset. This removes the divergence risk and is a prerequisite for a standalone
`server` entry point. It also retro-fixes a latent httpmock/cli-vs-client
name-dedup mismatch; scope that as a small same-PR follow-up if cheap, else a
tracked item — the server work only needs to *introduce* `prepare` and consume it.

---

## 7. Wiring the feature flag (four layers)

1. **Generator setting + module shape (Finding 4).** Add to `GenerationSettings`
   (`lib.rs:79`):
   ```rust
   generate_server: bool,            // default false
   ```
   with `with_server(&mut self, bool) -> &mut Self` next to `with_interface`
   (`lib.rs:143`). The standalone entry point `Generator::server(spec, crate_path)`
   exists regardless (like `httpmock`/`cli`) and returns the **module body** — the
   trait, request/query structs, responders, and `{Api}Server` — **without** a
   wrapping `pub mod server` (mirroring `httpmock`, which returns `pub mod
   operations …` + the trait unwrapped, for the caller to place). Each consumer
   adds the wrapper exactly once, so there is no double-nest:
   - `generate_tokens`, when `generate_server` is set, appends
     `pub mod server { #body }` to the client output;
   - conformance includes the body file inside `#[cfg(test)] pub mod server {
     include!(…) }` (§7.4).
   Keeping server tokens in their own module also means the client output is
   byte-identical when the flag is off (golden stability).
2. **`progenitor-macro`.** Add a `server = (true|false)` key parsed in
   `crates/progenitor-macro/src/lib.rs` (alongside `interface`/`tags`,
   `:341-362`) → `settings.with_server(..)`. Gate it behind a `server` **cargo
   feature** on `progenitor-macro` and on the umbrella `progenitor` so it is
   opt-in. **Crucially:** enabling the feature does *not* add a dependency on
   `progenitor-server` to `progenitor`/`progenitor-macro`; it only flips code
   emission. The user adds `progenitor-server` to their own `[dependencies]`
   (exactly the tonic-build/tonic split). Document this in the macro rustdoc
   (`lib.rs:89-146`).
3. **`cargo-progenitor`.** Add `--server` (emit the server module). When set,
   update the manual dependency table in `dependencies()` (`main.rs:240`) to inject
   **`progenitor-server = { version = …, features = ["transport"] }`** into the
   generated `Cargo.toml` `[dependencies]`, and append the server module to the lib
   output (`main.rs:180-186`). **Minimal-dep refinement (Finding 5):** that single
   dep suffices — the generated server names `axum`, `async_trait`,
   `serde_html_form`, etc. **only** through `progenitor_server::codegen::*` (the
   tonic `codegen` trick), so they arrive transitively and are *not* added as
   direct deps of the generated crate. The runtime is **never** vendored inline
   (unlike `progenitor_client.rs`) — too large (axum/tokio), always a real dep.
   **Server-only crates are deferred (Finding 6).** Today `--include-client=false`
   only swaps vendoring for a `progenitor-client` dep; it still emits the `Client`
   (`main.rs:176`). A genuine `--no-client --server` crate needs type emission
   factored out of client emission (a types-only generator), which doesn't exist
   yet. So v1 always emits client + types (optionally + server); do **not** ship a
   `--no-client` flag that silently still emits the client — omit it in v1 or land
   the types-only split first (tracked, §12 D-server-only).
4. **Conformance.** Extend `conformance_support::generate`
   (`support/src/lib.rs:50`) to also write `$OUT_DIR/server.rs` via a new
   `generate_server_helpers(gen, spec)` (sibling of `generate_mock_helpers`,
   `:199`), gated by a `spec.toml` field (e.g. `server = true`) or a cargo
   feature on the spec crate so only crates that opt in pay the axum compile
   cost. Spec crates that opt in add `progenitor-server` (+ `reqwest` for the
   raw-HTTP 501 probe) to `[dev-dependencies]`; **`tokio` comes via
   `conformance_support::tokio`** (already re-exported, `support/src/lib.rs:22`),
   and **no direct `axum`** dev-dep unless a test calls axum APIs directly
   (Finding 5). They `include!` the file inside `#[cfg(test)] pub mod server { … }`,
   exactly like the existing `mock` module (`conformance/anthropic/src/lib.rs:75-78`).

**Footprint guarantee to verify:** with the flag off, `cargo tree` for a
generated client shows no `axum`/`progenitor-server`; the client golden files are
unchanged. This is a hard acceptance criterion (§14).

---

## 8. Implementation phases

Ordered so each lands independently and is testable. Phase names are
S0, S0.5, S1…S9.

- **S0 — Design hardening (new; before writing the generator).** Prototype, by
  hand, the load-bearing shapes the generator will emit, so compile problems
  surface in hand-written code, not generated code:
  1. the `Metadata` `FromRequestParts` extractor + a handler that also takes a
     body extractor (Finding 3) — confirm it compiles and runs under axum 0.8;
  2. one per-op responder for each response shape — `Type`, `None`, `Raw`, and a
     hand-written **server-side** synth enum matched by variant (Findings 1, 2);
  3. the `Server`/`Bound`/`serve_listener` API against an ephemeral
     `127.0.0.1:0` listener (Finding 8);
  4. the `ServerError<E>` data type + per-op encoding ergonomics;
  5. a **repeated-array query** round-trip — serialize a `Vec` query param with the
     generated client and confirm the custom `progenitor_server::Query` extractor
     (serde_html_form) deserializes it (Finding 1).
  Output: a throwaway crate under `target/plan-scratch/` (not committed) pinning
  the exact patterns S1/S3/S4 must generate.
- **S0.5 — Contract tightening (new; lock the generated-surface contracts before
  the generator).** Write down, with test cases, the contracts an implementing
  agent could otherwise fill differently: the **module-body** shape and its single
  wrapping (§7.1); `prepare` returns **owned** `PreparedIr` (§6.9); the
  **status-resolution** rules + validation matrix (§6.4) as concrete cases —
  including **synth success + `with_status` on `Code`/`Range`/`Default`** and
  override-conflict→500; the **extraction-rejection** path (a malformed body yields
  the runtime's standardized 400, *not* axum's default short-circuit, §6.5); the
  **client-arm-order validator** (an error `Default` must not accept a
  success-claimed status, §6.4); and `Raw → bytes::Bytes` with no streaming
  (D-raw). These are decisions to pin, not code to ship.
- **S1 — Runtime crate skeleton.** Create `crates/progenitor-server`: `Request`/
  `Response`, `ServerError<E>` (**data only, no blanket `IntoResponse`**), the
  `Metadata` extractor, the repeated-key `Query` extractor (Finding 1),
  extractor rejections → **standardized HTTP responses** (runtime 400/415, *not*
  `ServerError<E>`; review 6 Finding 1), `codegen` re-exports, and the `transport`
  module (`Server`/`Router`/`Bound`, `serve`/`serve_listener`) behind the
  `transport` feature; `axum` **non-optional** (Finding 5). Unit-test the wrappers,
  both extractors (incl. repeated-key query), and the `bind → local_addr` path.
  Add `axum`/`async-trait`/`serde_html_form` to workspace deps; add the crate to
  `Cargo.toml:2-14`.
- **S2 — Generator IR plumbing.** Add `PathTemplate::as_axum_path` (+ test). Add
  `OperationResponse.media_type` and populate it in `process_operation` (Finding
  7). Factor `Generator::prepare(spec) -> PreparedIr` (Finding 4) and a helper
  that, from a prepared `OperationMethod`, returns the success/error **response
  items + payload types + default success status** (via the same `extract_responses`
  the client uses). **Also factor the shared status classifier** (review 6
  Finding 6) — the function that emits/evaluates the client's own arm patterns
  (`success_arm_pattern` etc., `method.rs:411-431`) and classifies a status as
  `Success`/`Error`/`Unexpected`; both the client's arm generation and the server
  validator (§6.4) consume it so they can't drift. All of this is shared by
  S3/S4 and reused by `generate_tokens`.
- **S3 — Emit trait + request structs + error aliases.** New
  `crates/progenitor-impl/src/server.rs`, `pub fn server(&mut self, spec,
  crate_path) -> Result<TokenStream>` built on `prepare`/`PreparedIr` (not a
  httpmock-style re-derive). Emit request structs (§6.3), the **server-side** synth
  enums where needed (§6.4), error aliases, and the `#[async_trait]` trait
  (§6.1, §5.3). Golden: a small spec's trait.
- **S4 — Emit the adapter, responders, and router.** Add `{Api}Server<T>`, the
  per-op `{op}_route` handlers (§6.5) **and** per-op `{op}_respond` responders
  (§6.4), `into_router`, and the `Service` impl. Handle JSON + path + query +
  simple headers + raw/text bodies; emit **501 route stubs** for skipped ops
  (upgrade/multipart/deepObject) — never silent drops (Finding 9). Golden update.
- **S5 — Settings + standalone wiring.** Add `generate_server`/`with_server`
  (§7.1); append the `server` module in `generate_tokens` when set. Confirm
  flag-off output is byte-identical (goldens).
- **S6 — Macro + cargo-progenitor.** `server` key/feature in the macro (§7.2);
  `--server` in the CLI + generated `Cargo.toml` dep injection (§7.3). **No**
  `--no-client` in v1 (Finding 6) unless the types-only split lands first.
- **S7 — Conformance integration.** `generate_server_helpers` +
  `$OUT_DIR/server.rs` (§7.4); opt-in flag in `spec.toml`; wire petstore-31 as
  the first server-enabled crate.
- **S8 — Round-trip test (the headline).** petstore-31: implement the trait with
  an in-memory store, `bind` an ephemeral port, point the **generated client** at
  `local_addr()`, assert typed success round-trips and a typed error decodes
  (§10).
- **S9 — Docs + example crate.** Add `crates/example-server` (mirrors
  `example-build`) and a README section; a second service on one `Server` to
  exercise multi-service mounting.

Each phase: run the user's `task` commands exactly as given (CLAUDE.md), keep
generated output prettyplease-formatted, regenerate goldens with
`EXPECTORATE=overwrite`, never write token-stream `.to_string()` to disk.

---

## 9. Crate & dependency layout / what is re-exported where

- **`progenitor-server`** (new, runtime): the only crate with `axum`/`tokio`/`tower`
  as **production (non-dev) dependencies**. Generated server code references it as
  `progenitor_server::*` and `progenitor_server::codegen::*`.
- **`progenitor-impl`** (generator): gains `src/server.rs` and the setting. No
  new runtime deps (it only produces tokens). It may gain `axum`/`progenitor-
  server` as **`[dev-dependencies]`** — compile-test only, to type-check golden
  server output; these never reach a consumer's dependency graph.
- **`progenitor`** (umbrella, used in build.rs/macro): gains an opt-in `server`
  cargo feature that flips macro emission. **Does not** depend on
  `progenitor-server`. Guarantee: a build.rs consumer that doesn't want a server
  pulls zero new deps.
- **`cargo-progenitor`**: learns `--server`; injects the `progenitor-server` dep
  into generated `Cargo.toml`; never vendors the runtime.
- **Consumer (e.g. luup2/Europace)**: to use a server, add
  `progenitor-server` to `[dependencies]` and enable `server` on their progenitor
  build/macro — two independent switches, like `tonic-build` + `tonic`.

---

## 10. Testing strategy

1. **Generator goldens.** Add a server-enabled fixture under
   `crates/progenitor-impl/tests/` (the `*_httpmock.rs` golden harness) so the
   trait/responders/adapter are reviewed and pinned. Cover every shape that can
   diverge: a path param, an optional query, a JSON body, a `201` success, a
   `None` (empty-body) success, a `Raw` body response, a **synth multi-kind**
   response, a typed default error, and a **skipped (upgrade) op → 501 stub**.
2. **Compile check.** The golden server output must compile against
   `progenitor-server` (dev-dep in `progenitor-impl`), catching drift between the
   response items and the generated responders/signatures.
3. **Round-trip conformance (S8) — the real proof.** In `conformance/petstore-31`
   (test cfg): implement `server::Petstore`, `bind` `127.0.0.1:0`, read
   `local_addr()` for the port (Finding 8), point the generated `Client` at it:
   - `create_pets` then `show_pet_by_id` returns the same `Pet` (typed in/out);
   - `list_pets` returns the seeded vec;
   - a not-found `show_pet_by_id` returns `ServerError::Api{404, types::Error}` and
     the **client** decodes it as `Error::ErrorResponse(ResponseValue<types::Error>)`
     — wire compatibility end to end.
   Reuses the tokio dev-dep posture (`conformance/anthropic/Cargo.toml`) and the
   `#[cfg(test)] pub mod` include convention.
4. **Encoding/edge unit tests** (handler level, mostly no socket): empty-body
   response emits no body + the right status; a raw response sets its content-type
   (once the IR carries it, §6.8); a present typed header round-trips; an
   **off-contract status override** (`with_status`/`Api.status` outside the declared
   set) → **500 + log** (§6.4 matrix); a synth response emits the active variant's
   status + body (not a JSON enum); a repeated-array query param (`?x=1&x=2`)
   round-trips client→server (Finding 1). **Runtime-standard rejections are
   asserted at the raw-HTTP / tower-service level, not via the typed client**
   (review 7 Finding 4 — they may not surface cleanly through it, same caveat as
   501): a malformed-JSON body → `400`/`415` and a missing-required header → `400`,
   each checked by hitting the `axum::Router` directly (`tower::ServiceExt::oneshot`)
   or over raw HTTP. A skipped op's **501 stub** is likewise asserted via raw HTTP
   — an empty 501 surfaces through the client as `InvalidResponsePayload` with
   `.status() == None` (Finding 2).
5. **Footprint test.** Assert (via `cargo tree`/a small check) that a
   server-disabled client crate has no `axum` in its tree and its client golden is
   unchanged.
6. **Multi-service.** Two `*Server` instances on one `progenitor_server::Server`,
   both reachable.
7. **Tooling tests.** (a) **Macro expansion** — a `progenitor::generate_api!` with
   `server = true` expands and compiles (trybuild/expand fixture). (b) **CLI dep
   injection** — `cargo-progenitor --server` emits a `Cargo.toml` containing
   `progenitor-server` (with `transport`) and *no* stray direct axum/serde_html_form
   deps, and the generated crate compiles (Finding 5).

---

## 11. Concrete file-change checklist (for the implementing agent)

- `Cargo.toml` (root): add `crates/progenitor-server` to members; add `axum`,
  `async-trait`, `serde_html_form` (and `tower`, `tower-http`) to
  `[workspace.dependencies]`.
- `crates/progenitor-server/`: new crate (`Cargo.toml`, `src/lib.rs`,
  `request.rs`, `response.rs`, `error.rs` [`ServerError<E>` data type, **no**
  blanket `IntoResponse`], `extract.rs` [`Metadata` + the custom request
  extractors with **runtime-standard** rejections: `Path`, `Query`
  (serde_html_form), and bodies `Json`/`Form`/`Bytes`/`Text` (each `Rejection` =
  runtime 400/415); plus `required_header::<T>()` / `optional_header::<T>()` parse
  helpers used in-route (Finding 4)], `service.rs` [the always-available `Service` trait, **not**
  gated by `transport`], `transport.rs` [behind `transport`:
  `Server`/`Router`/`Bound`/`serve`/`serve_listener`], `codegen.rs`). `axum`
  non-optional.
- `crates/progenitor-impl/src/server.rs`: new; `pub fn server(...)` consuming
  `PreparedIr`; emits the trait, request structs, server-side synth enums, per-op
  `{op}_respond` responders, `{Api}Server` + `into_router`, and 501 route stubs.
- `crates/progenitor-impl/src/method.rs`: add `OperationResponse.media_type` and
  populate it in `process_operation` (Finding 7); factor the **shared status
  classifier** out of the arm-pattern logic (`success_arm_pattern` etc.,
  `method.rs:411-431`) so client arm-gen and the server validator share it
  (review 6 Finding 6).
- `crates/progenitor-impl/src/lib.rs`: `mod server;`, factor
  `prepare(spec) -> PreparedIr` (Finding 4) and route `generate_tokens` through it;
  `generate_server: bool` in `GenerationSettings`, `with_server`, append `server`
  module in `generate_tokens` when set, re-export anything needed (mirror how
  `httpmock` is exposed).
- `crates/progenitor-impl/src/template.rs`: `as_axum_path` + test.
- `crates/progenitor-macro/src/lib.rs`: `server` key + feature → `with_server`.
- `crates/progenitor-macro/Cargo.toml`, `crates/progenitor/Cargo.toml`: `server`
  feature (no `progenitor-server` dep).
- `crates/cargo-progenitor/src/main.rs`: `--server` flag; **extend
  `dependencies()` (`main.rs:240`)** to inject `progenitor-server` (with the
  `transport` feature) — and *only* that, since axum/serde_html_form arrive via
  `progenitor_server::codegen` (Finding 5); append the server module to the lib
  output.
- `conformance/support/src/lib.rs`: `generate_server_helpers` + write
  `$OUT_DIR/server.rs`; `SpecManifest` gets a `server: bool` field
  (`conformance/support/src/fetch.rs`).
- `conformance/petstore-31/`: opt in (`spec.toml`, `Cargo.toml` dev-deps,
  `src/lib.rs` server module + round-trip tests).
- `crates/example-server/`: new example.
- Goldens: regenerate with `EXPECTORATE=overwrite`.

---

## 12. Open questions / decision register

Status (2026-06-13, after seven design reviews; reviews 6-7 were
consistency/precision passes with no new decisions): all load-bearing decisions are
**decided** — D-shape, D-async, D-error, D-runtime-feat, D-skip / D-server-only /
D-media-type, D-query / D-raw / D-501, D-crate-path / D-success-status,
D-extract-reject / D-synth-override, and **D-header**. The fifth review's
refinements: extraction failures are **runtime-standard** (not per-op typed,
D-extract-reject); error bodies follow the **response-kind table** (typed→JSON,
()→empty, Bytes→raw, §6.4); the status validator is a **shared mechanical
classifier** built from the client's own arm patterns (§6.4, §6.9); headers are
parsed **in-route** via `meta.headers()` + `required_header`/`optional_header` (§6.5); conformance
dev-deps are trimmed to `progenitor-server` + `reqwest` with `tokio` via the
support re-export (§7.4). Contracts are pinned in §6.4 (status resolution + matrix
+ classifier), §6.5 (custom extractors + in-route headers), §6.9 (owned `prepare`
+ shared classifier), §7.1 (module-body shape). Nothing is open; the plan is
implementation-ready **pending the S0/S0.5 prototype + contract lock**, which
de-risk the axum extractor/responder shapes, rejection handling, and the
repeated-key query before the generator is written.

| # | Decision | Resolution |
|---|---|---|
| D-shape | Trait method takes a **bundled `Request<{Op}Request>`** (A) vs **positional typed args + a metadata ctx** (B), mirroring the client method signature | **DECIDED — A** (2026-06-13, per the original request). One uniform `Request<T>` metadata wrapper; forward-compatible (adding a param doesn't change arity). B can't wrap N args in one generic `Request<T>`, so it's incompatible with the requested wrapper design. |
| D-error | **Universal `ServerError`** (tonic `Status` style) vs **per-op typed error** vs **hybrid** | **DECIDED — hybrid `ServerError<E>` as data + per-operation generated encoding** (2026-06-13, per review Finding 2). Typed `Api{status, E}` for declared errors (exact client interop) + `Internal`/`Response` escape hatch; encoding lives in generated `{op}_respond`, **not** a blanket `IntoResponse` (which can't handle `()`/raw/synth-status). See §6.4. |
| D-async | `#[async_trait]` (tonic's choice, dyn-friendly, simple) vs **native AFIT + `trait_variant`** (no per-call Box; MSRV 1.88 allows it) | **DECIDED — `#[async_trait]`** (2026-06-13, matches tonic; dodges `Send`-inference pitfalls with axum). Native AFIT + `trait_variant::make(.. : Send)` is noted as a later perf pass. The chosen macro is re-exported via `progenitor_server::codegen`. |
| D-tag | Single trait for the whole API vs per-tag traits | Single (v1). Add per-tag behind the existing `TagStyle::Separate` later, kept consistent with the client. |
| D-stream | Websocket `Upgrade` and SSE/streaming response bodies | Defer; skip-with-comment in v1. Raw `Bytes` request/response bodies are in v1. |
| D-deepobject | deepObject/exploded query params | Skip-with-`#[allow]`+TODO in v1 (httpmock already skips these); raw query reachable via the request parts. |
| D-runtime-feat | Feature split for the runtime crate | **DECIDED (Finding 5):** one crate, `axum` **non-optional**; `transport` (default on) gates only the tokio serving convenience (`Server`/`Bound`/`serve*`). With it off you still get wrappers + `into_router` + extractors. The earlier "off = wrappers + into_router only" was contradictory — `into_router` needs axum. |
| D-skip | Known-but-unsupported ops (upgrade/multipart/deepObject) | **DECIDED (Finding 9):** omit from the trait but emit a **501 route stub**, so routing is complete and clients see 501 not 404. Acceptance reads "one method per *supported* op." |
| D-server-only | A true server-only crate (no `Client`) | **Deferred (Finding 6):** needs type emission factored out of client emission (a types-only generator). v1 always emits client + types; no `--no-client` flag until that lands. |
| D-media-type | Response content-type fidelity for `Raw` | **DECIDED (Finding 7):** add `OperationResponse.media_type` (IR extension, S2); until populated, default `application/octet-stream` (raw) / `application/json` (typed). |
| D-query | How the server parses query params (esp. arrays) | **DECIDED (review 2, Finding 1):** a custom `progenitor_server::Query` extractor (repeated-key, via `serde_html_form`), **not** axum's `serde_urlencoded`-based `Query`, so it round-trips the client's repeated-key arrays (`QuerySeq`, `progenitor_client.rs:845-867`). |
| D-raw | `Raw` body/response payload type | **DECIDED (review 2):** `bytes::Bytes` in v1 — streaming is out of scope (D-stream). |
| D-501 | Is a skipped op's 501 cleanly observable via the generated client? | **DECIDED (review 2, Finding 2):** no — the client decodes the error body, so an empty/plain 501 → `InvalidResponsePayload` (`.status()` `None`). 501 stubs target *direct* HTTP callers; tests assert via raw HTTP and the generator emits a build warning listing skipped ops. |
| D-crate-path | How the server body references spec types | **DECIDED (review 3, Finding 1):** `use #crate_path::types;` (mirrors httpmock), rendering to `use crate::types;` in the common in-crate case; **never** `use super::types`. One API serves both in-crate and out-of-crate generation. |
| D-success-status | Validating success/error status against the contract | **DECIDED (review 3, refined reviews 4-6 Finding 2):** validate via the **shared mechanical classifier** (§6.4, §6.9) that replays the client's *own* emitted arm patterns and classifies a status by first match (`Success`/`Error`/`Unexpected`) — **not** a hand-modeled "success then error then Default" (`Default`/`Range` are embedded in the arms, `method.rs:411-431`). A success responder's status must classify `Success`, an error responder's `Error`. Off-contract → 500 + log; `ServerError::Response` is the escape hatch. |
| D-extract-reject | What an extraction failure returns | **DECIDED (review 4, refined review 5 Finding 1):** custom `progenitor_server` extractors (`Json`/`Form`/`Bytes`/`Text`/`Path`/`Query`) whose `Rejection` is a **runtime-standard** 400/415 — *not* the op's typed error (a generic extractor can't know it). The client may surface it as `InvalidResponsePayload`; raw HTTP sees the real status. Typed extraction errors are opt-in via `Result<_, Rejection>` + per-op mapping. |
| D-header | Typed-header extraction shape | **DECIDED (review 5, refined review 6 Finding 3):** parse **in the generated route** from `meta.headers()` via `progenitor_server::required_header::<T>(&meta, name) -> Result<T, Rejection>` and `optional_header::<T>(&meta, name) -> Result<Option<T>, Rejection>` — no name-less runtime extractor / no generated marker types. |
| D-synth-override | `with_status` on a synth success variant | **DECIDED (review 4, Finding 3):** the variant is the sole status source; `with_status` is accepted only if equal to the implied (`Code`) or carried (`Range`/`Default`) status, else 500 + log. |
| D-status | How to pick the default success status when several 2xx exist | Lowest concrete 2xx, else 200; always overridable via `Response::with_status`. |
| D-validate | Enforce OpenAPI `required`/format/pattern on the way in | Rely on serde + typed newtypes (typify already encodes many constraints). Optional richer 422 validation is a follow-up. |

---

## 13. Risks

- **axum version churn.** Path syntax changed at 0.7→0.8; pin `axum = "0.8"` and
  centralize the path renderer + extractor choices in one place so a future bump
  is a single edit. (Low — http/hyper are already 1.x in-tree.)
- **`async fn` Send-inference with axum.** axum handlers need `Send` futures;
  this is exactly why D-async starts with `async-trait`. (Mitigated by the
  decision.)
- **Compile-cost regression.** Pulling axum into a spec crate is heavy; the whole
  point of the opt-in flag + `[dev-dependencies]`/feature gating is to keep it off
  the default corpus path. The footprint test (§10.4) guards this. (Plan 01's
  measured frontend-bound costs mean we must not make axum a default dep.)
- **Response-encoding divergence (the headline risk, per review).** A "typed
  server" and a "wire-compatible client" quietly diverge if the server *encodes*
  responses differently than the client *decodes* — most dangerously `None` (empty
  body vs `null`), `Raw` content-type, and `Synth` (a status-tagged sum, not a
  JSON enum). Guardrails: the per-op generated responder owns encoding (§6.4); the
  shared `prepare`/response-item helper keeps both sides on one source (§6.9);
  goldens cover every shape (§10.1); the round-trip + encoding unit tests
  (§10.3-4) exercise them against the real generated client.
- **Query wire-compat (request-side analog, review 2).** The client encodes array
  query params as repeated keys (`QuerySeq`), which axum's built-in `Query`
  (serde_urlencoded) can't parse — a silent mismatch. Mitigated by the custom
  `progenitor_server::Query` extractor (§6.5, D-query) and the repeated-array
  round-trip test (§10.4); deepObject stays explicitly skipped in v1.
- **Spec messiness.** Operations with exotic content types / multiple bodies /
  no operationId. The IR already normalizes operationIds (`lib.rs:432-447`) and
  has graceful `Raw`/`Synth` fallbacks; the server mirrors the client's posture
  (skip-with-comment, never crash generation).

---

## 14. Acceptance criteria

1. With the server flag **off**, every existing client golden is byte-identical
   and no generated client crate gains `axum`/`progenitor-server` in `cargo tree`.
2. With the flag **on** for petstore-31: the crate compiles; the generated
   `server::Petstore` trait has one `async fn` per **supported** operation taking
   `Request<{Op}Request>` and returning `Result<Response<Success>, {Op}Error>`.
   Unsupported ops (upgrade/multipart) get a 501 route stub and **no** trait
   method (Finding 9).
3. A hand-written `impl server::Petstore` served via
   `progenitor_server::Server::builder().add_service(..).serve(addr)` answers the
   **generated client**'s calls: typed success round-trips, and a declared error
   returned as `ServerError::Api{status, body}` is decoded by the client as
   `Error::ErrorResponse(ResponseValue<…>)` (S8 test green).
4. `{Api}Server::new(impl).into_router()` returns a composable `axum::Router`
   (power-user path) and two services mount on one `Server` (multi-service test).
5. The implementor writes **zero** (de)serialization or header-parsing code for
   declared params/bodies/responses.
6. `cargo-progenitor --server` produces a crate depending on `progenitor-server`
   (never vendored) that compiles and emits client + types + server. (A true
   server-only `--no-client` crate is out of scope for v1 — D-server-only.)
7. Wire-compat is verified for every shape: response `None`→empty, `Raw`
   →passthrough, `Synth`→active-variant status+body, typed→JSON; and a
   repeated-array query param round-trips client→server (§10.4). A skipped op
   responds 501 to a **raw HTTP** probe (the typed client can't observe it cleanly
   — Finding 2).
8. All `task` lint/test commands pass; goldens regenerated and committed.