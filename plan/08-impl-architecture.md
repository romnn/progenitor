# 08 — progenitor-impl architecture: fixes, convergence, and hygiene

Architecture review of the generator core (`crates/progenitor-impl`, its seam
to `crates/typify-impl`, and the crate-level workspace config), performed
2026-07-10 on branch `feat/openapi3.1-support`. This document is the
implementation plan derived from that review. It does **not** overlap plans
01–07 (harness, corpus, layout, server): everything here is about the
generator's own code.

**Audience**: an implementer (human or model) who has not read this codebase.
Every item states the problem with `file:line` references, the exact fix, and
acceptance criteria. Line numbers are against the tree at the commit this
document was added; re-locate by the quoted identifiers if they have drifted.

## Verdict

**Sound shape, needs hygiene — plus two structural convergences.**

The big architectural decision of this fork is right: version-specific
frontends lower into a version-agnostic IR (`src/ir/mod.rs`), and every
backend (client methods, CLI, httpmock, server) consumes only the IR. The
dependency direction is clean — `openapiv3` types are confined to the 3.0
frontend; the public API (`parse_openapi_str` → opaque `OpenApiDocument`)
leaks nothing. The parsing/repair layer (`openapi.rs`, `openapi/ref_repair.rs`)
is well-factored and unusually well-commented. That hill is the right hill;
do not re-architect it.

The two places where the design fights itself:

1. **Schema lowering exists three times** and is kept byte-identical only by
   convention and twin tests: `to_schema.rs` (920 lines, typed
   `openapiv3::Schema` → schemars draft-07), `ir/v31/schema.rs`
   (`SchemaLowering`, raw-Value 2020-12 → draft-07), and the hybrid-document
   normalizers in `openapi.rs` (which rewrite 3.1 spellings *back into* 3.0
   spellings so the strict `openapiv3` parser accepts them). Part B converges
   on one path.
2. **The backend layer's shared vocabulary lives inside one backend.**
   `OperationMethod` and its satellite types — consumed by all four emitters —
   are defined in `method.rs` (3,260 lines, at least seven distinct reasons to
   change), and two backends (`cli()`, `httpmock()`) bypass the shared
   `prepare()` pipeline, which is a real output bug (A1). Part C extracts the
   model and unifies the pipeline.

Separately, the workspace's strict lint policy is **configured but applied to
zero crates** (Part E), and there is a short list of generation-time panics
reachable from spec input (Part A).

## Ground rules for whoever implements this

- **Behavior-preserving commits.** Structural change and behavioral change
  never share a commit. Every item below says whether golden output is
  allowed to change; when it says "golden-neutral", a golden diff means the
  refactor is wrong — do not regenerate goldens to make the test pass.
- **Verification per commit**: `task test` (workspace unit + golden tests) and,
  before finishing a Part, `task test:conformance`. Run these exactly as
  written (task runner commands are configured; do not substitute raw cargo).
- Golden regeneration, when an item explicitly allows it:
  `EXPECTORATE=overwrite cargo test -p progenitor-impl`, then **read the
  diff** and confirm it matches the item's predicted change before committing.
- Commit subjects: lowercase imperative with a conventional prefix
  (`fix(impl): …`, `refactor(impl): …`). No `Co-Authored-By` or
  "Generated with" trailers.
- `serde_json`'s `preserve_order` feature is load-bearing (see workspace
  `Cargo.toml` comment). Nothing here may reorder document walks, map
  insertions, or schema emission order.
- If an item's fix collides with something this plan didn't predict (an
  unexpected golden diff, a test that encodes the old behavior), stop that
  item, note what you found, and move on rather than forcing it.

---

## Part A — Correctness fixes (do these first; each is one small commit)

Ranked by realism of the failure. All are independent of Parts B–F.

### A1. `cli()` and `httpmock()` silently degrade on a fresh `Generator`

**Problem.** `Generator::prepare` (`src/lib.rs:435-471`) is the shared
pipeline: `add_ref_types` + the `schema_supertypes`/`schema_type_ids` maps +
the `process_operation` walk + operation-id dedup. `Generator::cli`
(`src/cli.rs:24-38`) and `Generator::httpmock` (`src/httpmock.rs:31-45`)
re-implement only the first and third steps and **skip the maps and the
dedup**. But `cli_method` calls `extract_responses` (`src/cli.rs:205-208`),
which reads `self.schema_supertypes` (`src/method.rs:1825`) — empty unless the
*same* `Generator` instance previously ran `prepare()` via
`generate_tokens`/`generate_text`.

**Failure scenario.** Call `generator.cli(&spec, ...)` on a fresh `Generator`
(a legitimate public-API use): the allOf-ancestor response collapse silently
doesn't happen, so CLI output differs from what the same spec produces after a
client generation on the same instance. Likewise, two raw operation IDs that
sanitize to the same snake-case name are deduplicated in the client but not in
CLI/httpmock output → generated code with duplicate method names that fails to
compile. Today's golden tests mask this because `tests/test_output.rs:75-98`
reuses the generator that already generated the tagged-builder client.

**Fix.**
1. In `cli()` and `httpmock()`, delete the local `add_ref_types` +
   `process_operation` walks and instead call `let prepared =
   self.prepare(spec)?;`, then iterate `prepared.raw_methods`.
   (`prepare` is documented idempotent for already-registered names —
   `src/lib.rs:433-434` — so the existing reuse-after-client flow keeps
   working.)
2. Both functions currently walk `document.operations` and call
   `process_operation` per op; `prepared.raw_methods` is exactly that result,
   post-dedup.
3. Add a regression test in `tests/test_specific.rs`: generate CLI output for
   a spec with an allOf-sibling response pattern once on a fresh generator
   and once on a generator that first ran `generate_tokens`; assert the two
   outputs are identical.

**Golden impact:** none expected (the golden flow already had `prepare` run).
If a golden diff appears, the fresh-path behavior was live somewhere — stop
and report.
**Effort:** small (≤1 h).

### A2. Operation-id dedup in `prepare()` can still produce collisions

**Problem.** The dedup at `src/lib.rs:455-468` renames the *n*-th occurrence
of a sanitized name to `{name}_{n}` without checking whether that name is
already taken. `OperationMethod.operation_id` is
`sanitize(operation_id, Case::Snake)` (`src/method.rs:927`), so distinct raw
IDs can collapse.

**Failure scenario.** Raw IDs in document order `foo`, `foo_2`, `Foo`:
sanitized `foo`, `foo_2`, `foo` → the third becomes `foo_2`, colliding with
the second. Generated client fails to compile with duplicate method names.
(Note `ir::ensure_operation_ids` in `src/ir/mod.rs:268-290` dedups **raw** IDs
correctly with a taken-set; this pass handles post-sanitization collapse and
must use the same taken-set approach.)

**Fix.** Replace the count map with a `HashSet<String>` of taken names; on
collision, bump a numeric suffix until free (mirror
`ir::ensure_operation_ids`'s loop). Unit-test the `foo`/`foo_2`/`Foo` case.

**Golden impact:** none for the sample specs (no such collisions). **Effort:**
small.

### A3. Function-pointer identity used as a discriminator

**Problem.** `extract_responses` (`src/method.rs:1762-1768`) recovers whether
it was asked for the success side or the error side by comparing its filter
argument `as *const ()` against `is_success_or_default as *const ()`. Rust
does not guarantee stable function-pointer identity (functions can be merged
or duplicated across codegen units); this is a latent miscompile-shaped bug.
The inner function (`extract_responses_inner`, `src/method.rs:1772-1777`)
already takes an explicit side parameter — the outer wrapper reconstructs it
heuristically just to keep a two-argument signature.

**Fix.** Change `extract_responses` to take the explicit side enum instead of
a filter function; enumerate its callers (`src/method.rs`, `src/cli.rs:205-208`,
`src/server.rs:179-182`) and pass the side each already implies. If some
caller passes a filter that is neither of the two named classifiers, keep the
filter parameter but *add* the explicit side parameter — never infer it from
the pointer.

**Golden impact:** none. **Effort:** small.

### A4. Generation-time `todo!`/`panic!` reachable from spec input

Each of these aborts the whole generation for a document shape a wild spec
can legally (or sloppily) contain. The fork's stated goal is wild-spec
tolerance; these should degrade or error, never panic. Convert one per commit:

1. `src/to_schema.rs:581` — `todo!("invalid type: {}")` on an unrecognized
   `type` string. Only unreachable because `normalize_type_strings`
   (`src/openapi.rs:255-281`) pre-cleans input — a cross-file convention
   nothing enforces. Change to degrade: treat the unrecognized type like an
   untyped schema (drop the constraint), matching how `type: null` is already
   handled at `src/to_schema.rs:575-579`. (Part B deletes this file; do this
   one-liner anyway — B may be deferred.)
2. `src/method.rs:1514-1518` — `todo!()` for non-default upgrade (websocket)
   error responses. Return
   `Err(Error::UnexpectedFormat("upgrade operations with non-default error responses are not supported: …"))`.
3. `src/cli.rs:283-284` — `todo!()` for paginated Raw/Upgrade CLI output.
   Skip the operation in CLI output instead (emit nothing for it), the same
   graceful-degrade policy the server backend uses for unsupported ops.
4. `src/method.rs:2993, 2997, 3036` — `sort_params` panics when a path
   template names a parameter the operation doesn't declare (and vice versa)
   and on a duplicate body. Real wild specs hit exactly this class of typo.
   Make `sort_params` return `Result` and convert the panics to
   `Error::InvalidPath` / `Error::UnexpectedFormat`; `process_operation`
   already returns `Result`.
5. `src/to_schema.rs:703` — `serde_json::Number::from_f64(*value).unwrap()`.
   Not reachable from JSON input today (JSON has no non-finite numbers), but
   free to fix: fall back to skipping the value.

For each: add a minimal inline test (a `json!` document exercising the shape)
asserting `parse_openapi_*`/generation returns an error or degrades instead of
panicking. **Golden impact:** none. **Effort:** small each.

---

## Part B — One schema-lowering path (structural; staged; biggest drift-killer)

### The problem, concretely

A schema reaches typify (which only accepts schemars 0.8 draft-07 — its whole
input surface is `add_type`/`add_ref_types`/`add_root_schema` over
`schemars::schema::Schema`, `typify-impl/src/lib.rs:617,811,844`) through
**three separately-maintained converters**:

| Path | Code | Direction |
|---|---|---|
| 3.0 documents | `src/to_schema.rs` (~920 lines) | typed `openapiv3::Schema` → schemars |
| 3.1 documents | `src/ir/v31/schema.rs` (`SchemaLowering`, ~600 lines logic) | raw `Value` 2020-12 → draft-07 `Value` → schemars |
| 3.1 idioms inside 3.0 docs | `src/openapi.rs:336-436` (`normalize_nullable_type_unions` etc.) | rewrites 2020-12 spellings *back to* 3.0 so `openapiv3` parses them |

The same semantic decisions are encoded in all of them — nullable scalars,
nullable composites (`oneof_nullable_wrapper` at `to_schema.rs:736-766` vs
`wrap_nullable` at `schema.rs:524-548`, whose doc literally says "Replicate
`to_schema.rs`'s `oneof_nullable_wrapper`"), draft-4 exclusive bounds
(**three** sites: `to_schema.rs:167-174,202-211,477-502`,
`schema.rs:265-284`, and `openapi.rs:359-370` rewriting in the *opposite
direction*), discriminator stashing (`to_schema.rs:123-135` vs
`schema.rs:226-231`), example folding (`to_schema.rs:117` vs
`schema.rs:236-252`). The byte-equality contract between them is stated in a
module doc (`schema.rs:12-16`) and enforced only by twin tests
(`schema.rs:602-808`, `tests/test_v31.rs`).

**Drift scenario.** Every schema-semantics change must be hand-mirrored in
two (sometimes three) places or 3.0/3.1 twins silently diverge — the exact
class of bug the twin tests were built to catch, encoded as a permanent tax.
And the 3.0 path is *strict*: `openapiv3` rejects sloppiness, so every new
wild-spec quirk needs another pre-normalizer in `openapi.rs` (there are five
already). The tolerant 3.1 path absorbs the same quirks natively.

### Why convergence is cheap now

`SchemaLowering` **already handles the 3.0 spellings** because hybrid wild
documents forced it to: `nullable: true` (`schema.rs:296-311`), boolean
exclusive bounds (`schema.rs:265-284`), `const`→enum (`schema.rs:258-263`),
nullable `$ref` via the same oneOf wrapper (`schema.rs:313-326`), singular
`example` (`schema.rs:236-252`), discriminator stash (`schema.rs:226-231`).
The document-structure skeleton (`ir/v31/mod.rs`) is likewise nearly
version-agnostic (fixed method order, null-entry tolerance, final-segment ref
resolution — all already mirror 3.0 semantics). What remains is an audit, a
differential gate, and deletion.

### Staged plan

**B0 — differential harness (no production change).** Add a test (e.g.
`tests/test_frontend_convergence.rs`) with a helper that takes a 3.0 document
`Value`, runs it through (a) the existing `v30` path and (b) the `v31` path
with the version check bypassed, and asserts `generate_text` output is
byte-identical. Wire it over every 3.0 spec in `sample_openapi/` and (behind
an env guard like the existing slow gates) the 3.0 half of the conformance
corpus. Expect failures initially — they are the work list for B1.

**B1 — close the gaps.** Read `to_schema.rs` top-to-bottom; for each behavior,
tick off: already in `SchemaLowering` / needs adding / artifact of the typed
AST (drop). Add the missing rewrites to `SchemaLowering` (keyed on a new
`Dialect::V30 | V31` parameter **only if** a keyword's meaning actually
differs by dialect; prefer dialect-agnostic handling where the spellings are
disjoint, which is most of them). Grow the twin tests in `schema.rs` for each
gap closed. Iterate until B0 is green corpus-wide.

**B2 — flip the dispatch.** In `parse_openapi_value`
(`src/openapi.rs:194-217`), route 3.0 documents to the unified frontend.
Keep the old path compiled and the differential test comparing both until a
full conformance run (`task test:conformance`) and the golden suite are green.
**Gate: generated output must be byte-identical** — this whole Part is
golden-neutral by definition; any diff is a B1 gap.

**B3 — delete.** Remove `src/to_schema.rs`, `src/ir/v30.rs`, the `openapiv3`
half of `src/util.rs` (`ReferenceOrExt`/`ComponentLookup`/`items`/
`parameter_map`, `util.rs:6-103`), `normalized_openapiv3` +
`normalize_nullable_type_unions` + `strip_null_path_entries` in
`src/openapi.rs` (keep the dialect-agnostic sloppiness normalizers:
`normalize_type_strings`, `normalize_response_descriptions`,
`ensure_info_version`, `normalize_non_query_deep_object_parameters`, and
`ref_repair`). Move `openapiv3` to `[dev-dependencies]` (tests still use it,
e.g. `ir/mod.rs:417-432`, the twin harness at `schema.rs:625-627`). Rename
`ir/v31` to reflect that it is now *the* frontend (e.g. `ir/frontend`), update
the module docs that describe the two-frontend world (`ir/mod.rs:3-15`).

**Abort criteria.** If B1 uncovers a semantic dependency on `openapiv3`'s
typed AST that cannot be expressed as a Value rewrite (none is known; the
closest is `AnySchema` handling, which the twin tests already pin), stop and
write up the residue instead of forcing it.

**Effort:** the largest item in this plan — roughly B0 half a day, B1 one to
two days, B2+B3 half a day. Deletes ~1,300 lines and one dependency, and turns
the byte-equality convention into a tautology.

---

## Part C — Backend layer: shared model out of `method.rs`, one pipeline

### C1. Extract the operation model (do first; unblocks the rest)

`OperationMethod` (`src/method.rs:21-32`), `HttpMethod` (`:34-75`),
`OperationParameter`/`OperationParameterType`/`OperationParameterKind`
(`:100-143`), `BodyContentType` (`:146-254`),
`OperationResponse`/`OperationResponseStatus`/`OperationResponseKind`
(`:256-395`), `DropshotPagination` (`:95-98`), plus the pure classifiers
(`is_success_or_default`/`is_error_or_default` `:320-337`,
`synth_variant_name` `:401-407`, `sort_key` `:304-317`) are the shared
vocabulary of **all four** backends (`server.rs:26-29`, `cli.rs:12`,
`httpmock.rs:10-14`) but live in the client-method emitter file.

**Fix.** Move them verbatim to a new `src/operation.rs` ("the resolved
operation model" — the backend-facing IR). Fix imports in the four backends.
Pure mechanical move; golden-neutral. **Effort:** small.

### C2. Split the rest of `method.rs` by reason-to-change (opportunistic)

After C1, `method.rs` still holds: spec→model lowering (`process_operation`
`:509-938`, `get_body_param` `:2707-2820`, `sort_params` `:2975-3057`, schema
predicates `:2846-2890`); response-set analysis (`extract_responses` +
collapse passes `:1747-1877`, `find_common_supertype` `:165-194`,
`dropshot_pagination_data` `:1881-1996`); shared request codegen
(`method_sig_body` `:1169-1710`, synth enum/arms `:451-507,1717-1745`); the
positional emitter (`:948-1164`); the builder emitter (`:2081-2705`); and doc
rendering (`:2892-2973`). The clusters share almost no private state — they
communicate via `OperationMethod` and `Generator` fields — so the split is
mechanical:

```
src/operation.rs          (C1: the model + classifiers)
src/operations/lower.rs   process_operation, get_body_param, parameter_schema,
                          sort_params, has_content_keywords, is_plain_string_schema
src/operations/responses.rs  extract_responses*, find_common_supertype,
                          collapse_bodyless_with_typed, dropshot_pagination_data
src/emit/method.rs        method_sig_body, synth_* emission, doc-comment helpers
src/emit/positional.rs    positional_method + its pagination stream
src/emit/builder.rs       builder_struct/impl/tags/helper + its pagination stream
```

Do it one file per commit, golden-neutral each. Do **not** further split
typify or invent traits between `Generator` and the emitters — every emitter
has exactly one implementation and no test/swap seam justification.
**Effort:** medium, mechanical.

### C3. Make deep-object query params unrepresentable-wrong

`deep_object_query: bool` is bolted onto `OperationParameter`
(`src/method.rs:110`) beside a `Query(bool)` kind whose bool means "required"
(`:119-139`). The IR already models this properly
(`ir::QueryStyle::DeepObject`, `src/ir/mod.rs:113-118`); lowering flattens it
into the bool at `:581`, and every backend must then *remember* to check it —
server both skips such ops (`server.rs:191`) **and** keeps a defensive
re-check (`server.rs:731-737`); cli (`cli.rs:382`) and httpmock
(`httpmock.rs:140-144`) each have their own guard.

**Fix.** Change the kind to carry it:
`OperationParameterKind::Query { required: bool, deep_object: bool }` (and
give `Header`/`Cookie` named `{ required: bool }` fields while touching it).
Delete the standalone field; let the compiler enumerate every match site;
remove the server's defensive re-check once the type carries the fact.
Golden-neutral. **Effort:** small-medium (compiler-driven).

### C4. Thread `PreparedIr` instead of hidden `Generator` state

`schema_supertypes`, `schema_type_ids`, and `component_schemas` are
`Generator` fields populated by `prepare()` (`src/lib.rs:62-76,435-471`) and
read later by `extract_responses` and `example_schemas` — a call-order
invariant enforced by doc comments ("Must be called after generate_text…",
`src/lib.rs:397-399`; "callers MUST have run prepare", `server.rs:63-65`),
which A1 shows already got violated once.

**Fix.** Grow `PreparedIr` to own the three maps alongside `raw_methods`;
`extract_responses` and the emitters take `&PreparedIr` explicitly (the
codebase already threads `raw_methods` this way for exactly the borrow
reasons documented on `PreparedIr`, `src/lib.rs:78-85`). Keep the public
signatures of `generate_tokens`/`cli`/`httpmock`/`server`/`example_schemas`
unchanged: each entry point calls `prepare()` itself (after A1 they all do),
and `prepare` may additionally stash the copies `example_schemas` reads.
Golden-neutral. **Effort:** medium.

### C5. Stop mirroring the response classifier into the server backend

`server.rs` re-implements the client's status classification and synth
naming: `response_status_predicate` (`server.rs:1091-1126`, doc: "mirrors the
client-side response classifier"), `server_synth_variant_ident`
(`server.rs:1136-1143`) duplicating `synth_variant_name`
(`method.rs:401-407`), plus `HttpMethod`→token maps written three times
(`method.rs:62-75,1603-1610`, `server.rs:990-1002`, `httpmock.rs:122-131`).

**Fix.** After C1, the shared pieces live in `operation.rs`: give
`HttpMethod` the token-mapping methods; export `synth_variant_name` and the
status predicates; delete the server/httpmock copies (keep genuinely
server-local policy like the `Response`-suffix naming, `server.rs:1128-1134`).
Golden-neutral. **Effort:** small.

### C6. Compute parameter optionality once (optional, drift-reduction)

Lowering unwraps `Option<T>` params three times inside `process_operation`
(`method.rs:596-601,628-635,675-682`) and then throws the fact away, so
server (`server.rs:876-946`), cli (`cli.rs:561-573`), and httpmock
(`httpmock.rs:160-172,368-375`) each re-derive "is this optional, and what's
the inner type" from the `TypeId`. Similarly the Display-fallback degrade is
implemented three near-identical times in `process_operation`
(`method.rs:554-564,642-655,684-697`) and differently again in httpmock
(`httpmock.rs:189-201`) and server (`server.rs:937-941`).

**Fix.** Extend the model so `OperationParameter` carries
`{ type_id, optional: bool, inner_type_id: Option<TypeId> }` computed once at
lowering; migrate backends off their local re-derivations one at a time.
Golden-neutral if done faithfully; verify per backend. **Effort:** medium.
Lower priority than C1–C5.

---

## Part D — The discriminator seam between progenitor and typify

**Problem.** `x-discriminator` is a stringly cross-crate protocol: produced in
two places (`to_schema.rs:129-135`, `ir/v31/schema.rs:226-231`), consumed by
name inside typify (`typify-impl/src/lib.rs:1202`). And the consumer —
`discriminator_to_oneof_prepass`, ~490 lines at
`typify-impl/src/lib.rs:1087-1579` — is OpenAPI-specific logic living inside
the JSON-Schema→Rust library, growing this fork's divergence from upstream
typify (which matters: this typify is itself a maintained fork).

**D1 (cheap, do it).** Export a `pub const DISCRIMINATOR_EXTENSION_KEY` and a
small serde struct for the extension payload (`propertyName`, `mapping`) from
`typify-impl`, and use them at both producer sites and the consumer. One
spelling, one shape. Golden-neutral. **Effort:** small.

**D2 (optional, decide before doing).** Move the prepass into
`progenitor-impl`, running over `ir::Document::schemas` (it is a
schema-map→schema-map rewrite) before `add_ref_types`
(`src/lib.rs:438-443`). Benefit: typify returns to being a pure JSON-Schema
tool and the fork's upstream diff shrinks by ~490 lines + tests. Risk: the
prepass currently sees typify's `RefKey` view of definitions; the move must
preserve exact ref-name semantics (`#/components/schemas/`, `#/definitions/`,
`#/` prefixes, `typify-impl/src/lib.rs:1258-1261`). Gate on goldens +
conformance being byte-identical. **Effort:** medium-high. Skip if upstream
typify syncing is not a near-term goal.

---

## Part E — The lint policy is configured but enforced nowhere

**Problem.** The workspace defines a strict deny-set
(`[workspace.lints]` in the root `Cargo.toml`: pedantic, `unwrap_used`,
`panic`, `indexing_slicing`, …) and `clippy.toml`'s comment asserts
"unwrap/expect/panic are denied in production code (workspace lints)" — but
**no member crate declares `[lints] workspace = true`**, so per Cargo
semantics the table applies to nothing. Verified: `cargo clippy` passes
cleanly while `progenitor-impl` alone contains ~100 `unwrap`/`expect` calls
in non-test code. The safety net everyone believes exists, doesn't.

**Fix, staged (one crate per commit, run `task lint` after each):**

1. **Runtime crates first** — `progenitor-client`, `progenitor-server`: add
   `[lints] workspace = true`; fix every violation properly (these crates
   ship inside user binaries; panic-freedom is a real product property here).
2. **Small tool crates** — `progenitor-macro`, `cargo-progenitor`: same.
3. **`progenitor-impl`**: add the inheritance, then run `task lint` and count.
   Expect a large pedantic burden plus the unwrap set. Land it as: fix
   everything cheap; for the residual internal-invariant unwraps (e.g.
   `type_space.get_type(id).unwrap()` where the id was just created), add
   per-lint overrides in the crate's own `[lints]` table
   (`unwrap_used = "allow"` etc.) with a comment explaining they are
   generation-time invariants, and ratchet down in follow-ups. Do **not**
   blanket-allow pedantic; fix or per-lint-override each specific lint.
   Panics reachable from *spec input* are Part A items regardless.
4. **Leave `typify`/`typify-impl`/`typify-macro` and the `example-*` crates
   out** deliberately (fork-divergence cost / throwaway examples) and say so
   in a comment next to the workspace lints table.
5. Fix the `clippy.toml` comment to describe whatever reality this lands on.

**Effort:** steps 1–2 small; step 3 is a grind — timebox it and keep the
overrides honest.

---

## Part F — Small hygiene (any order, one commit each, all golden-neutral)

1. **`util.rs` is two modules glued together**: openapiv3 reference
   resolution (`util.rs:6-103`, used only by `ir/v30.rs`) + generic
   identifier/doc helpers (`sanitize`, `unique_ident_from`,
   `neutralize_doc_fences`). Move the openapiv3 half into `ir/v30.rs` (or a
   `ir/v30/` submodule). Subsumed by B3 — do it only if Part B is deferred.
2. **`crate_path` parsed three times** with three panic messages
   (`server.rs:72-73`, `cli.rs:63-66`, `httpmock.rs:61-65`) → one
   `util::parse_crate_path(&str) -> Result<syn::Path>`.
3. **Dropshot pagination magic strings** `"page_token"`/`"limit"` matched by
   name at `method.rs:1043-1082,1893-1897,2371-2373` and `cli.rs:396` →
   named constants next to `DropshotPagination` in the (post-C1) model
   module.
4. **Stale comment**: the pop-trailing-default pass
   (`method.rs:1795-1807`) justifies itself by "the multi-distinct-kind
   assert below", which no longer exists. Update the comment to state the
   real current rationale (signature stability of the collapsed response
   type) while touching this code in A3.
5. **`eprintln!` as the only channel for skipped server operations**
   (`server.rs:204-212`): collect skip notices on the `Generator`
   (`Vec<String>` + public accessor) so build.rs/macro consumers can surface
   them; keep the eprintln as a default sink.
6. **`builder_struct` iterates `method.params` five times** building parallel
   arrays (`method.rs:2091-2187`) — collapse to a single pass producing one
   per-param struct. Readability only; do it opportunistically when C2
   touches the file.
7. **IR fields kept for the future** carry bare `#[allow(dead_code)]`
   (`src/ir/mod.rs:94-97,129-130,143-144`) → `#[expect(dead_code, reason =
   "…")]` so the suppression self-expires when the field gains a reader.

---

## What NOT to do

- Do not restructure `typify-impl`'s `convert.rs`/`merge.rs`/`type_entry.rs`
  — upstream-sync cost outweighs any internal tidiness gain.
- Do not introduce traits/seams between `Generator` and the backends, or
  ports-and-adapters layering anywhere: every component has one
  implementation and the crate boundary (`progenitor-impl` vs `typify`) is
  the only real seam.
- Do not add newtypes speculatively (e.g. wrapping every `String` name); the
  type-budget items in this plan (C3, C4, D1) are the ones paying rent.
- Do not change generated output in any refactor commit. The goldens are the
  invariant, not a formality.
- Do not renumber or edit plans 01–07; cross-reference them instead.

## Suggested sequencing

| Order | Item(s) | Why this order |
|---|---|---|
| 1 | A1–A4 | Real bugs; tiny; independent |
| 2 | C1 | Unblocks C2/C5/F3; pure move |
| 3 | C3, C5, F2–F5, D1 | Small, compiler-driven, independent |
| 4 | C4 | Builds on A1; kills the call-order invariant |
| 5 | E1–E2 (runtime + small crates) | Bounded; high product value |
| 6 | B0→B3 | The big convergence; needs a quiet stretch |
| 7 | C2, C6, F6 | Opportunistic splits after the dust settles |
| 8 | E3, D2 | Grinds / optional; decide explicitly before starting |

Parts B and C are independent of each other; if Part B lands first, F1 and
the `to_schema.rs` items in Part A shrink accordingly.
