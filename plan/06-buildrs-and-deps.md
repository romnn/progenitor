# 06 — build.rs simplification and dependency consolidation

Request: (a) the `syn::parse2::<syn::File>` in build.rs "is unnecessary because
if it would generate invalid code cargo check would already fail right?";
(b) "make generate_tokens() already do prettyplease formatting to slim down
build.rs"; (c) `load_manifest("qdrant")`-style lookup; (d) "consolidate the
deps needed … in a helper crate so that the conformance tests need only the
smallest number of deps possible".

## 1. Answering (a) and (b) precisely

- (a) Correct: if build.rs wrote unparseable text, the consumer's own
  compilation of the `include!`d file would fail anyway. The parse in build.rs
  buys exactly two things: a *clear* error at generation time (vs. rustc
  pointing into a giant OUT_DIR file) and the `syn::File` needed for
  formatting. Both are preserved for free by `generate_text`, which already
  does tokens → `syn::parse2` (mapped to `Error::InternalError("generated
  code does not parse: …")`) → `prettyplease::unparse`
  (progenitor-impl/src/lib.rs:368-373). So: no explicit parse in any build.rs.
- (b) `generate_tokens` itself *cannot* format — it returns a
  `proc_macro2::TokenStream`, which carries no whitespace or layout; formatting
  only exists once tokens are rendered to text. The API answer is already in
  the tree: **`generate_text` is the formatted entry point**; the plan is to
  make every text-writing call site use it, not to change `generate_tokens`
  (which the proc-macro path needs as tokens).

Call sites that currently hand-roll the tokens→syn→prettyplease triplet and
should switch to `generate_text` (each then drops `syn` + `prettyplease` from
its `[build-dependencies]`):

- the 12 conformance build.rs — replaced wholesale by the 3-line
  `conformance_support::generate("spec.toml")` form in 05 step 2 (the support
  crate uses `generate_text` internally),
- `example-build/build.rs` (also drops its unused `serde_json` build-dep),
- `example-wasm/build.rs` (also: its progenitor build-dep pulls default
  features = the whole macro stack; add `default-features = false`),
- `cargo-progenitor`'s `reformat_code` (main.rs:90-93) — replace with
  `generate_text` for the lib.rs path (the CLI/httpmock token streams it also
  emits still need a local parse+unparse helper, so `reformat_code` may stay
  for those — but it should be the only remaining manual site).

## 2. Spec lookup (c): superseded by per-crate spec.toml

The original ask — `load_manifest("qdrant")` instead of the load-then-find
dance (16 call sites today) — is answered more radically by 05: there is no
central manifest to look up *into*. Each spec crate carries its own
`spec.toml` and its build.rs passes that file to `generate` explicitly.
`load_manifest`/`SpecEntry`/`manifest.toml` are deleted with
`crates/wild-tests`; the inventory view is a `conformance/*/spec.toml` glob
(used by the `generate_all` smoke and the `new-spec` scaffolder).

Why a spec.toml at all, instead of writing the URL inline in build.rs (the
maximally inline option)? Because the smoke test, the scaffolder, CI sharding,
and the D4 sha256 pinning all need a statically readable registry — inline
Rust strings would force tooling to parse build scripts. The explicit-file
design keeps both properties: build.rs is obvious about its input, and the
input is machine-readable data.

## 3. The support crate

One new crate, `conformance/support/` (package `conformance-support`, a member
of the conformance workspace), with two faces:

**Build face** (used from `[build-dependencies]`):

```rust
/// Drives build.rs generation from an explicit spec manifest (a path relative
/// to the crate root — cargo runs build scripts with cwd = the package root).
/// Reads it (name, url= or file=, optional sha256/notes), asserts `name`
/// matches the crate directory, emits cargo:rerun-if-changed for the manifest
/// and the cached spec, fetches url-sourced specs (shared cache at
/// conformance/support/cache/<name>.<ext>; CONFORMANCE_REFRESH=1 re-downloads),
/// parses, runs Generator::generate_text, and writes $OUT_DIR/codegen.rs.
/// Panics with actionable messages (incl. "populate the cache or allow
/// network" on offline cache misses).
pub fn generate(spec_manifest: impl AsRef<Path>);
```

Every spec crate's build.rs becomes:

```rust
fn main() {
    conformance_support::generate("spec.toml");
}
```

3 lines, one build-dependency, no per-crate name strings to typo in code (the
audit found two copy-pasted wrong names in the current 12 build.rs; here the
name lives once, in spec.toml, and is asserted against the directory).
Hermetic crates use `file = "petstore-31.json"` in spec.toml instead of
`url =` — no separate `generate_from_path` entry point; the source kind is
explicit data, not a second API. Internally the support crate depends on
progenitor with `default-features = false` — the existing conformance
workspace already demonstrates that trims the macro stack out of build deps.
It also hosts the `generate_all` smoke test and the `new-spec` scaffolder bin
(05 §2).

**Test face** (used from `[dev-dependencies]` of generated/spec crates):

- the T2 assertion macros from 03 (`roundtrip!`, `assert_off_wire!`,
  `assert_wire_enum!`, `assert_union_variants!`, `assert_rejects!`);
- re-exports for operation tests: `pub use httpmock; pub use tokio;
  pub use serde_json;` — attribute paths like
  `#[conformance_support::tokio::test]` resolve fine through a re-export, so
  spec crates need no direct tokio/httpmock deps.

## 4. Runtime deps: why a helper crate alone cannot shrink them — and what can

The 10 runtime deps every conformance/generated crate carries (bytes, chrono,
futures-core, percent-encoding, regress, reqwest, serde, serde_json,
serde_urlencoded, uuid) exist because **generated code references those crates
by absolute path** (`::serde::…`, `::chrono::…`) — and `::serde` resolves
against the *consumer's* dependency graph. Re-exporting serde from a helper
does nothing for `::serde` paths. Two levels of consolidation:

**Level 1 (mechanical, lands with 05/06):** workspace-level dep inheritance +
the support crate for build/test faces. Per-crate Cargo.toml ends up ~12
lines: `[build-dependencies] conformance-support`, `[dev-dependencies]
conformance-support`, `[dependencies]` = progenitor-client + the runtime deps
as `{ workspace = true }` one-liners. This is the floor without generator
changes.

**Level 2 (decision D5, the real consolidation): the progenitor-client
facade.** Make `progenitor-client` re-export the runtime surface
(`pub use reqwest; pub use serde; pub use serde_json; pub use chrono;
pub use uuid; pub use regress; pub use bytes; pub use futures_core;` — chrono/
uuid/regress behind features mirroring typify's `uses_*` introspection, base64/
rand behind the websocket feature), and teach the generators to emit paths
through it. Audited touchpoints:

- progenitor-impl side is cheap: the client body mostly emits *bare*
  `reqwest::` / `futures::` paths already, resolvable by adding
  `use progenitor_client::reqwest;` (etc.) to the emitted module header —
  after normalizing the ~10 absolute-path stragglers in method.rs
  (`::reqwest::Method::OPTIONS`, websocket `::base64`/`::rand`, the
  `::futures::` trio; the file is currently inconsistent bare-vs-absolute,
  which this cleans up anyway).
- typify side needs a "runtime crate prefix" setting touching a known, finite
  site list (audited): the derive-string pair
  (type_entry.rs:893-894 — becomes `"::progenitor_client::serde::Serialize"`…),
  a `#[serde(crate = "::progenitor_client::serde")]` container attr pushed at
  the three container sites (enums type_entry.rs:972-974, structs :1296-1297,
  newtype `#[serde(transparent)]` :1839), the two bespoke
  `impl ::serde::Deserialize` blocks (:1688, :1790), the regress LazyLock
  (:1735-1736), `::serde_json` Map/Value (:1936, :2012) plus the
  `skip_serializing_if = "::serde_json::Map::is_empty"` string attr
  (structs.rs:422) and default-value `::serde_json::from_str` (value.rs:175,
  181), and the chrono/uuid type-name strings (convert.rs:903, 914, 924).
  Note `#[serde(crate)]` redirects only derive-*generated* code; the derive
  macro path and the string attrs need their own repointing — that is exactly
  what the site list covers.
- End state: a consumer (conformance crate, luup2 build.rs consumer,
  cargo-progenitor output) needs **`progenitor-client` as its only runtime
  dependency**. cargo-progenitor's hand-maintained `DEPENDENCIES` table —
  which the audit caught drifting *today* (rand "0.8" vs workspace 0.10,
  regress "0.10" vs 0.11, bytes "1.9" vs 1.11) — collapses to one line,
  eliminating that drift class permanently.
- Costs, honestly: every golden regenerates (mechanical; expectorate
  overwrite + careful diff review); progenitor-client's version then pins the
  reqwest/serde majors for consumers (already de-facto true — generated code
  hard-codes their APIs); consumers constructing field values write
  `progenitor_client::chrono::DateTime<…>` or add chrono themselves (same
  crate either way, both compile).
- **Spike before committing (1 day):** wire the facade through one golden
  (keeper) + one monster (stripe types only): confirm derive-macro re-export
  paths work in `#[derive(...)]` position, `#[serde(crate)]` composes with
  `untagged`/`tag`/`transparent`/`flatten`/`deny_unknown_fields` as used in
  our output, and `cargo doc` link resolution doesn't regress. Then D5 go/no-go.

## 5. Bundled hygiene (do with whichever workstream touches the file first)

- wild-tests' unused `expectorate` dev-dep disappears with the crate (05).
- example-macro/example-out-dir: no changes needed beyond 04 path moves;
  example-wasm `default-features = false` on progenitor (above).
- conformance/discord's dead `serde_json` build-dep and the github-31/openai
  expect-message typos die when 05 step 2 replaces the build.rs bodies and
  Cargo.toml dep lists.

## 6. Acceptance criteria

- No build.rs in the repo contains `syn::parse2` / `prettyplease::unparse`;
  every text-writing generation path goes through `generate_text`.
- A new spec crate needs: 1 build-dependency, 1 dev-dependency, and
  `[dependencies]` of progenitor-client + runtime one-liners (Level 1) — or
  progenitor-client alone (Level 2, if D5 approved).
- cargo-progenitor emits Cargo.tomls whose dependency versions cannot drift
  from the workspace (either derived from one source of truth, or facade).
- luup2's five build.rs consumers can drop to the same minimal dep set when
  they bump their pin (verify against `~/dev/branches/luup2` once D5 lands).
