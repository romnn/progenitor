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
an `#[async_trait]` **service trait** with one `async fn` per operation taking a
typed, wrapped request and returning a typed, wrapped response/error, and (b) a
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
enforcement. These degrade gracefully (skip-with-log or `unimplemented!` stub),
exactly like httpmock skips the synthesized multi-kind responses today
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
   paths can fold it into the primary output. It emits a `pub mod server { … }`
   containing the trait, the per-op request structs, the per-op error enums, and
   the `{Api}Server<T>` → `axum::Router` adapter. Emitted **only** when enabled;
   zero axum/server footprint otherwise.
2. **Generated code shape** (§6): trait + wrappers + router wiring, referencing
   the runtime as `progenitor_server::*` (never inlined; see §9).
3. **Runtime crate `progenitor-server`** (§6.6): `Request<T>`, `Response<T>`,
   `ServerError`, extractor/response glue, `codegen` re-export module
   (`async_trait`, `axum`, `http`, …), and the `Server`/`Router` convenience.

Flag plumbing (§7) threads one switch through `GenerationSettings` →
`progenitor-macro` (`server` config key + cargo feature) → `cargo-progenitor`
(`--server`/`--no-client`) → `conformance_support::generate` (emit
`$OUT_DIR/server.rs`).

---

## 5. Worked example (petstore-31) — the target output

Given `GET /pets?limit`, `POST /pets` (body `Pet`, 201 → `Pet`), `GET
/pets/{petId}` (200 → `Pet`, default → `Error`), the server module should look
like this (abbreviated; `types::*` are the **same** types the client uses):

```rust
pub mod server {
    use super::types;
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
        // one extractor->call->responder per op (see §6.5 for the body)
        async fn list_pets_route(/* State, Query, … */) -> axum::response::Response { /* … */ }
        async fn create_pets_route(/* State, Json<Pet> */) -> axum::response::Response { /* … */ }
        async fn show_pet_by_id_route(/* State, Path<String> */) -> axum::response::Response { /* … */ }
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
  (e.g. `Petstore`), in a `pub mod server`. Adapter `{Trait}Server<T>`. This
  matches the default `TagStyle::Merged` and the single-`Client` client today.
- Method name = `operation_id` after the same dedup that
  `generate_tokens` applies (`lib.rs:432-447`), so trait methods line up 1:1
  with client methods.
- Trait bound `Send + Sync + 'static` (axum handlers are spawned and shared).
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
    parts: http::request::Parts,   // headers, method, uri, extensions, version
}
impl<T> Request<T> {
    pub fn new(message: T) -> Self;                 // tests/fixtures
    pub fn get_ref(&self) -> &T;
    pub fn get_mut(&mut self) -> &mut T;
    pub fn into_inner(self) -> T;
    pub fn headers(&self) -> &http::HeaderMap;
    pub fn extensions(&self) -> &http::Extensions;  // ConnectInfo, auth results
    pub fn into_parts(self) -> (http::request::Parts, T);
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

`Request<T>` carries `http::request::Parts` rather than a bespoke `MetadataMap`
(REST headers are just `http::HeaderMap`; no need to reinvent tonic's ASCII/bin
metadata split). This is the "generic wrapper carrying metadata" the request
asked for, with declared params already pulled into the typed `message`.

### 6.3 The per-operation request struct (the typed message `T`)

Generated per operation by walking `method.params`
(`OperationParameterKind`, `method.rs:120`):

- `Path` → field typed by the param's `TypeId` (string-ish; `String` when
  `RawBody`/opaque). Required.
- `Query(required)` → field; `Option<T>` when `!required`.
- `Header(required)` → field; `Option<T>` when `!required`.
- `Body(content_type)`:
  - `Json` / `FormUrlencoded` with `Type(id)` → `pub body: <typed>`.
  - `OctetStream` / `Raw` → `pub body: bytes::Bytes`.
  - `Text` → `pub body: String`.
- Field names reuse the param's sanitized `name`; the wire name (`api_name`) is
  what extraction binds to (mirrors the client and the httpmock matcher, which
  already track both — `httpmock.rs:146-153`).
- Derive `Debug, Clone`. **No `Serialize`/`Deserialize` on the bundle** — the
  bundle is assembled by the adapter from per-source axum extractors, not
  deserialized as a whole (query and body have different wire encodings). Typed
  sub-parts that *are* deserialized (the body type, a typed query struct) get
  their derives from typify already.

Design alternative (positional args, mirroring the client method signature) is in
§12 D-shape; recommendation is the bundle (Design A) for fidelity to the
"generic wrapper" requirement and forward-compatibility.

### 6.4 Success and error typing — reuse `extract_responses`

The trait's return type **must** match what the client decodes, so derive both
sides from the same place:

- Success type: `Generator::extract_responses(method,
  OperationResponseStatus::is_success_or_default)` (the call the client uses at
  `method.rs:1284`). The `Response<T>`'s `T` is that success type
  (`Vec<Pet>`, `Pet`, `()` for `None`, `bytes::Bytes` for `Raw`).
- Default success **status code**: the lowest concrete 2xx in the success set
  (e.g. `201` for create), falling back to `200`. Stored as a generated const so
  the adapter applies it when `Response.status` is `None`.
- Error type: `extract_responses(method, is_error_or_default)`
  (`method.rs:1341`). This is the **same typed error** the client surfaces via
  `Error::ErrorResponse(ResponseValue<E>)`.

Error model (the key decision — see §12 D-error). **Recommended:** a runtime
generic `ServerError<E>`:

```rust
pub enum ServerError<E> {
    /// A declared, typed error response: serialized as JSON with `status`.
    Api { status: http::StatusCode, body: E },
    /// Escape hatch for infra/unexpected failures (→ 500 by default).
    Internal(BoxError),
    /// Fully custom HTTP response (raw body + status + headers).
    Response(http::Response<axum::body::Body>),
}
impl<E: serde::Serialize> ServerError<E> {
    pub fn api(status: http::StatusCode, body: E) -> Self;
    pub fn internal(e: impl Into<BoxError>) -> Self;
}
impl<E: serde::Serialize> axum::response::IntoResponse for ServerError<E> { … }
```

This keeps errors **typed** (so client/server interop is exact for declared
errors) while giving an untyped `Internal`/`Response` path for the long tail —
the REST analog of tonic returning `Status` for anything. The per-op alias
(`type ShowPetByIdError = ServerError<types::Error>;`, §5.2) names it. Synthesized
multi-kind error sets (`OperationResponseKind::Synth`) reuse the same synthesized
enum the client emits, so `E` is always a real type.

`Response<T>` and the success/error halves all flow into one generated axum
responder per op (next section).

### 6.5 The adapter `{Api}Server<T>` → `axum::Router` (boilerplate lives here)

Per operation, generate one `async fn {op}_route` used as the axum handler. It is
where serialization/header parsing is centralized so the implementor never sees
it. Shape (for `show_pet_by_id`):

```rust
async fn show_pet_by_id_route(
    axum::extract::State(inner): axum::extract::State<Arc<T>>,
    axum::extract::Path(pet_id): axum::extract::Path<String>,
    parts: http::request::Parts,            // metadata source
    // (typed query via Query<…>, body via Json<…>/Bytes/String as applicable)
) -> axum::response::Response {
    let message = ShowPetByIdRequest { pet_id };
    let request = progenitor_server::Request::from_parts(parts, message);
    match <T as Petstore>::show_pet_by_id(&inner, request).await {
        Ok(resp) => progenitor_server::into_axum_ok(resp, /*default status*/ 200u16),
        Err(err) => axum::response::IntoResponse::into_response(err),
    }
}
```

Extraction rules (all standard axum extractors; **order matters** — body-
consuming extractor last):
- Path params: a single `Path<T>` or `Path<(T1, T2, …)>` tuple in path order.
- Query: a generated `#[derive(Deserialize)] struct {Op}Query { … }` extracted
  via `axum::extract::Query`. Optional fields use `Option<T>`. deepObject/exploded
  styles that the client handles specially (the httpmock generator skips these,
  `httpmock.rs:140-144`) get the same skip-with-`#[allow]` treatment in v1, with
  a `// TODO deepObject` and raw access via `request.uri().query()`.
- Headers: typed declared headers pulled from `parts.headers` by `api_name` and
  parsed via `FromStr`/serde; missing required → `400` through `ServerError`.
- Body by `BodyContentType`: `Json<T>` (json), `axum::extract::Form<T>`
  (urlencoded), `Bytes` (octet-stream/raw), `String` (text). A rejected body
  (malformed JSON, wrong content-type) becomes a `400`/`415` via the runtime's
  rejection→`ServerError` mapping, never reaching the trait.
- `into_axum_ok(resp, default_status)`: serialize `T` to JSON (or pass through
  `Bytes`/`()`), apply `resp.status` or the default, copy `resp.headers`.

Routing: group operations by **path template** and register methods on each
`axum::routing::MethodRouter` (`.get(..).post(..)`), then `.with_state(Arc<T>)`.
Path template translation in §6.7.

### 6.6 The runtime crate `progenitor-server`

New `crates/progenitor-server` (workspace member; runtime-only; **not** a build
dep). Modules:

- `request` / `response` / `error`: the types in §6.2 / §6.4.
- `extract`: helpers + the rejection→`ServerError` conversions, and any shared
  typed-header parsing.
- `codegen`: a `pub use` surface the generated code imports via
  `use progenitor_server::codegen::*;` — re-exports `async_trait::async_trait`,
  `axum`, `http`, `std::sync::Arc`, `bytes::Bytes`, serde traits. (Exactly tonic's
  `tonic::codegen::*` trick, `tonic/Cargo.toml:125`.) This keeps generated code
  terse and lets us swap implementation details without regenerating consumers.
- `transport`: the convenience server.
  ```rust
  pub trait Service { fn into_router(self) -> axum::Router; }

  pub struct Server { /* layers, timeouts, body limit, … */ }
  impl Server {
      pub fn builder() -> Self;
      pub fn concurrency_limit(self, n: usize) -> Self;
      pub fn timeout(self, d: Duration) -> Self;
      pub fn layer<L>(self, layer: L) -> Self;          // tower layer
      pub fn add_service<S: Service>(self, svc: S) -> Router;   // → Router
  }
  pub struct Router { /* merged axum::Router + Server config */ }
  impl Router {
      pub fn add_service<S: Service>(self, svc: S) -> Self;     // merge another
      pub fn into_make_service(self) -> /* axum */;
      pub async fn serve(self, addr: SocketAddr) -> Result<(), Error>;
      pub async fn serve_with_shutdown<F: Future<Output=()>>(self, addr, signal)
          -> Result<(), Error>;
  }
  ```
  `add_service` merges `svc.into_router()` into the accumulated `axum::Router`
  (`Router::merge`), applies configured tower layers, and `serve` binds a
  `tokio::net::TcpListener` and runs `axum::serve`. This is the tonic
  `Server::builder().add_service(..).serve(addr)` shape (`transport/server/
  mod.rs:511,979,1021`) adapted to axum. Multiple services = multiple specs (or
  per-tag traits) on one socket.

Features on `progenitor-server`: `default = ["transport"]`; `transport` pulls
`axum`/`tokio`/`tower(-http)`; turning it off leaves only the wrapper types +
`into_router` for users who bring their own server. (Mirrors tonic's
`transport`/`server`/`router` feature split.)

Cargo deps: `axum = "0.8"`, `http`, `bytes`, `serde`, `serde_json`,
`serde_urlencoded`, `async-trait`, `tokio` (rt-multi-thread, net),
`tower`, `tower-http` (optional), `futures`, `thiserror`. Add `axum` and
`async-trait` to the **root** `[workspace.dependencies]`.

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
  per §6.5. Multipart → v1 skips the op with a logged note (same posture as the
  `Raw(..)` fallback the client already uses, `method.rs:148-156`).
- Response kinds (`OperationResponseKind`, `method.rs:346`): `Type` → JSON;
  `None` → empty body + status; `Raw` → `Bytes` passthrough with the response's
  content-type; `Synth` → JSON of the synthesized enum (reuse the client's enum);
  `Upgrade` → **skip in v1** (websockets, §12). Skipped ops are omitted from the
  router and the trait, with a `// progenitor: skipped <op> (<reason>)` comment so
  the gap is visible — never a silent drop.

---

## 7. Wiring the feature flag (four layers)

1. **Generator setting.** Add to `GenerationSettings` (`lib.rs:79`):
   ```rust
   generate_server: bool,            // default false
   ```
   with `with_server(&mut self, bool) -> &mut Self` next to `with_interface`
   (`lib.rs:143`). The standalone entry point `Generator::server(spec,
   crate_path)` exists regardless (like `httpmock`/`cli`); the setting only
   controls whether `generate_text`/`generate_tokens` *also* append the server
   module. Keep server tokens in their own `pub mod server` so the client output
   is byte-identical when the flag is off (golden-test stability).
2. **`progenitor-macro`.** Add a `server = (true|false)` key parsed in
   `crates/progenitor-macro/src/lib.rs` (alongside `interface`/`tags`,
   `:341-362`) → `settings.with_server(..)`. Gate it behind a `server` **cargo
   feature** on `progenitor-macro` and on the umbrella `progenitor` so it is
   opt-in. **Crucially:** enabling the feature does *not* add a dependency on
   `progenitor-server` to `progenitor`/`progenitor-macro`; it only flips code
   emission. The user adds `progenitor-server` to their own `[dependencies]`
   (exactly the tonic-build/tonic split). Document this in the macro rustdoc
   (`lib.rs:89-146`).
3. **`cargo-progenitor`.** Add `--server` (emit server module) and let
   `--include-client=false` produce a server-only crate. When `--server`, add
   `progenitor-server` to the generated `Cargo.toml` `[dependencies]`
   (`main.rs:159-167`) and append the server module to the lib output
   (`main.rs:180-186`). The runtime is **never** vendored inline (unlike
   `progenitor_client.rs`) — it is too large (axum/tokio) and always a real dep.
4. **Conformance.** Extend `conformance_support::generate`
   (`support/src/lib.rs:50`) to also write `$OUT_DIR/server.rs` via a new
   `generate_server_helpers(gen, spec)` (sibling of `generate_mock_helpers`,
   `:199`), gated by a `spec.toml` field (e.g. `server = true`) or a cargo
   feature on the spec crate so only crates that opt in pay the axum compile
   cost. Spec crates that opt in add `progenitor-server` (and `axum`, `tokio`)
   to `[dev-dependencies]` and `include!` the file inside `#[cfg(test)] pub mod
   server { … }`, exactly like the existing `mock` module
   (`conformance/anthropic/src/lib.rs:75-78`).

**Footprint guarantee to verify:** with the flag off, `cargo tree` for a
generated client shows no `axum`/`progenitor-server`; the client golden files are
unchanged. This is a hard acceptance criterion (§14).

---

## 8. Implementation phases

Ordered so each lands independently and is testable. Phase names are S1…S9.

- **S1 — Runtime crate skeleton.** Create `crates/progenitor-server` with
  `Request`/`Response`/`ServerError`, `codegen` re-exports, and `IntoResponse`
  impls. No generator changes yet. Unit-test the wrappers and error→response
  mapping. Add `axum`/`async-trait` to workspace deps. Add the crate to
  `Cargo.toml:2-14` members.
- **S2 — Path + type plumbing in the generator.** Add `PathTemplate::as_axum_path`
  (+ test). Add a private `Generator` helper that, given an `OperationMethod`,
  returns `(success_type, default_success_status, error_type)` by calling the
  existing `extract_responses` the same way the client does — shared by S3/S4.
- **S3 — Emit trait + request structs + error aliases.** New file
  `crates/progenitor-impl/src/server.rs` with `pub fn server(&mut self, spec,
  crate_path) -> Result<TokenStream>` modeled on `httpmock.rs:31-114`. First
  emit only the request structs (§6.3), error aliases (§6.4), and the
  `#[async_trait]` trait (§6.1, §5.3). Golden: a small spec's trait.
- **S4 — Emit the adapter + router.** Add `{Api}Server<T>`, the per-op
  `{op}_route` handlers (§6.5), `into_router`, and the `Service` impl. Handle
  JSON + path + query + simple headers + raw/text bodies; skip-with-comment for
  upgrade/multipart/deepObject. Golden update.
- **S5 — Settings + standalone wiring.** Add `generate_server`/`with_server`
  (§7.1); append the `server` module in `generate_tokens` when set. Confirm
  flag-off output is byte-identical (goldens).
- **S6 — Macro + cargo-progenitor.** `server` key/feature in the macro (§7.2);
  `--server`/`--no-client` in the CLI (§7.3); generated `Cargo.toml` dep
  injection.
- **S7 — Conformance integration.** `generate_server_helpers` +
  `$OUT_DIR/server.rs` (§7.4); opt-in flag in `spec.toml`; wire one small spec
  (petstore-31) as the first server-enabled crate.
- **S8 — Round-trip test (the headline).** In the petstore-31 conformance crate,
  implement the generated trait with an in-memory store, run it on an ephemeral
  port via `progenitor_server::Server`, point the **generated client** at it, and
  assert client calls succeed and decode (incl. a typed error). See §10.
- **S9 — Docs + example crate.** Add `crates/example-server` (mirrors
  `example-build`) and a README section. Optionally a second service in the same
  `Server` to exercise multi-service mounting.

Each phase: run the user's `task` commands exactly as given (CLAUDE.md), keep
generated output prettyplease-formatted, regenerate goldens with
`EXPECTORATE=overwrite`, never write token-stream `.to_string()` to disk.

---

## 9. Crate & dependency layout / what is re-exported where

- **`progenitor-server`** (new, runtime): the only crate that depends on
  `axum`/`tokio`/`tower`. Generated server code references it as
  `progenitor_server::*` and `progenitor_server::codegen::*`.
- **`progenitor-impl`** (generator): gains `src/server.rs` and the setting. No
  new runtime deps (it only produces tokens). It may gain `axum`/`progenitor-
  server` as **`[dev-dependencies]`** to compile-check golden server output.
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
   `crates/progenitor-impl/tests/` (the same harness that produces
   `*_httpmock.rs` goldens) so the trait/adapter output is reviewed and pinned.
   Cover: a path param, an optional query, a JSON body, a 201 success, a typed
   default error, and one skipped (upgrade) op.
2. **Compile check.** The golden server output must compile against
   `progenitor-server` (dev-dep in `progenitor-impl`), catching type drift
   between `extract_responses` and the trait signatures.
3. **Round-trip conformance (S8) — the real proof.** In `conformance/petstore-31`
   (test cfg): implement `server::Petstore`, serve on `127.0.0.1:0`, take the
   bound port, build the generated `Client` against it, and:
   - `create_pets` then `show_pet_by_id` returns the same `Pet` (typed in,
     typed out);
   - `list_pets` returns the seeded vec;
   - a not-found `show_pet_by_id` makes the server return `ServerError::Api{404,
     types::Error}` and the **client** decodes it as
     `Error::ErrorResponse(ResponseValue<types::Error>)` — proving wire
     compatibility end to end.
   This reuses the existing tokio/httpmock dev-dep posture
   (`conformance/anthropic/Cargo.toml` pattern) and the `#[cfg(test)] pub mod`
   include convention.
4. **Footprint test.** Assert (via `cargo tree`/a small CI check) that a
   server-disabled client crate has no `axum` in its tree and its client golden
   is unchanged.
5. **Multi-service.** A test mounting two `*Server` instances on one
   `progenitor_server::Server` and hitting both.

---

## 11. Concrete file-change checklist (for the implementing agent)

- `Cargo.toml` (root): add `crates/progenitor-server` to members; add `axum`,
  `async-trait` (and `tower`, `tower-http`) to `[workspace.dependencies]`.
- `crates/progenitor-server/`: new crate (`Cargo.toml`, `src/lib.rs`,
  `request.rs`, `response.rs`, `error.rs`, `extract.rs`, `transport.rs`,
  `codegen.rs`).
- `crates/progenitor-impl/src/server.rs`: new; `pub fn server(...)`.
- `crates/progenitor-impl/src/lib.rs`: `mod server;`, `generate_server: bool` in
  `GenerationSettings`, `with_server`, append `server` module in
  `generate_tokens` when set, re-export anything needed (mirror how `httpmock`
  is exposed).
- `crates/progenitor-impl/src/template.rs`: `as_axum_path` + test.
- `crates/progenitor-macro/src/lib.rs`: `server` key + feature → `with_server`.
- `crates/progenitor-macro/Cargo.toml`, `crates/progenitor/Cargo.toml`: `server`
  feature (no `progenitor-server` dep).
- `crates/cargo-progenitor/src/main.rs`: `--server` flag; dep injection; module
  append.
- `conformance/support/src/lib.rs`: `generate_server_helpers` + write
  `$OUT_DIR/server.rs`; `SpecManifest` gets a `server: bool` field
  (`conformance/support/src/fetch.rs`).
- `conformance/petstore-31/`: opt in (`spec.toml`, `Cargo.toml` dev-deps,
  `src/lib.rs` server module + round-trip tests).
- `crates/example-server/`: new example.
- Goldens: regenerate with `EXPECTORATE=overwrite`.

---

## 12. Open questions / decision register

Status (2026-06-13): **D-shape** and **D-async** are **decided** — both follow
directly from the original request ("typed request wrapped in a generic metadata
wrapper" → bundled `Request<T>`; "copy tonic as much as possible" → `#[async_trait]`).
**D-error** is the one remaining load-bearing call awaiting Roman's input; the rest
have safe defaults an implementing agent can take as-is.

| # | Decision | Resolution |
|---|---|---|
| D-shape | Trait method takes a **bundled `Request<{Op}Request>`** (A) vs **positional typed args + a metadata ctx** (B), mirroring the client method signature | **DECIDED — A** (2026-06-13, per the original request). One uniform `Request<T>` metadata wrapper; forward-compatible (adding a param doesn't change arity). B can't wrap N args in one generic `Request<T>`, so it's incompatible with the requested wrapper design. |
| D-error | **Universal `ServerError`** (tonic `Status` style, untyped body) vs **per-op typed error** vs **hybrid `ServerError<E>`** | **OPEN — needs Roman's call.** Recommend the **hybrid `ServerError<E>`** (§6.4): typed `Api{status, E}` for declared errors (exact client interop) + `Internal`/`Response` escape hatch. This is the only open decision that changes the trait signature, the runtime crate, and the round-trip test. |
| D-async | `#[async_trait]` (tonic's choice, dyn-friendly, simple) vs **native AFIT + `trait_variant`** (no per-call Box; MSRV 1.88 allows it) | **DECIDED — `#[async_trait]`** (2026-06-13, matches tonic; dodges `Send`-inference pitfalls with axum). Native AFIT + `trait_variant::make(.. : Send)` is noted as a later perf pass. The chosen macro is re-exported via `progenitor_server::codegen`. |
| D-tag | Single trait for the whole API vs per-tag traits | Single (v1). Add per-tag behind the existing `TagStyle::Separate` later, kept consistent with the client. |
| D-stream | Websocket `Upgrade` and SSE/streaming response bodies | Defer; skip-with-comment in v1. Raw `Bytes` request/response bodies are in v1. |
| D-deepobject | deepObject/exploded query params | Skip-with-`#[allow]`+TODO in v1 (httpmock already skips these); raw query reachable via the request parts. |
| D-runtime-feat | One `progenitor-server` with a `transport` feature vs split crates | One crate, `transport` feature on by default; off = wrappers + `into_router` only. |
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
- **Type drift between client and server.** Both must derive types from
  `extract_responses`; the S2 shared helper + the round-trip test (S8) are the
  guardrails.
- **Spec messiness.** Operations with exotic content types / multiple bodies /
  no operationId. The IR already normalizes operationIds (`lib.rs:432-447`) and
  has graceful `Raw`/`Synth` fallbacks; the server mirrors the client's posture
  (skip-with-comment, never crash generation).

---

## 14. Acceptance criteria

1. With the server flag **off**, every existing client golden is byte-identical
   and no generated client crate gains `axum`/`progenitor-server` in `cargo tree`.
2. With the flag **on** for petstore-31: the crate compiles; the generated
   `server::Petstore` trait has one `async fn` per operation taking
   `Request<{Op}Request>` and returning `Result<Response<Success>, {Op}Error>`.
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
   (never vendored) that compiles; `--no-client --server` yields a server-only
   crate.
7. All `task` lint/test commands pass; goldens regenerated and committed.
```