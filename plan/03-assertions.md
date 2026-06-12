# 03 — Assertions: prove the generated code is right at runtime

Request: "for all the conformance tests, lets add more assertions to make sure
they won't fail at runtime and generate correct types and code!"

## 1. Where assertion coverage stands today (audited)

- **conformance/ (12 crates, 34 `#[test]`s total)**: all tests are inline
  `#[cfg(test)]` modules in src/lib.rs; all sync serde/type-shape assertions
  on `serde_json::json!` literals. No `tests/` dirs, no fixtures, **no
  httpmock, no tokio, no dev-dependencies at all** — the only client-level
  coverage is `Client::new(url)` + `baseurl()`. Three crates are
  constructor-smoke-only: anthropic, github-31, openai (1 test each).
- **wild spec-tests (7 files)**: rich behavioral suites for the six
  formerly-broken specs (clickhouse-cloud 8 tests, cloudflare 8,
  codat-accounting 9, discord 12, mongodb-atlas 8, pagerduty 8), but only
  clickhouse-cloud and discord run under plain `WILD_COMPILE=1` (the rest are
  slow-gated). `petstore-31.rs` is **orphaned** — it references a manifest
  entry that never existed, so it is never copied or compiled.
- **Doctests**: run on generated code in 9/12 conformance crates (a real
  regression net — the ``` fence neutralization was found this way). The
  emitted wild crates set `doctest = false` (embedded progenitor_client
  module), so doctest coverage exists *only* via conformance today.
- The review pass that found 14 bugs proved the thesis: generate+compile green
  hides wire bugs. Assertions are the ratchet that keeps them fixed.

This workstream assumes 05 (unification) lands first: every spec is a
`conformance/<name>/` crate, its assertions live in its own
`src/lib.rs` `#[cfg(test)]` module, and doctests on generated code run
corpus-wide (the real progenitor-client dep removes the old reason to disable
them).

## 2. T1 — auto-generated example round-trips (highest leverage; scales to 120)

Vendors ship examples in their specs; use them as free fixtures.

- `conformance_support::generate` walks the spec for example values while it
  generates: media-type `example`/`examples`, schema-level `example` (3.0) and
  `examples` (3.1), parameter examples. Progenitor already knows the
  schema→generated-type mapping (the `Generator`'s type space); emit
  `$OUT_DIR/example_tests.rs` next to `codegen.rs`, one `#[test]` per
  (type, example), and have lib.rs pull it in with
  `#[cfg(test)] mod example_tests { include!(concat!(env!("OUT_DIR"), "/example_tests.rs")); }`.
- Assertion shape: **idempotent round-trip**, not byte equality —
  `deserialize(example)` → `serialize` → `deserialize` and assert the two
  deserialized values are equal (`PartialEq` is available where needed thanks
  to the value-constraint work; fall back to comparing re-serialized
  `serde_json::Value`s). Byte equality is wrong here: generated types
  materialize schema defaults and omit unset optionals, so serialization
  legitimately differs from the raw example.
- **Ratchet for vendor mistakes**: wild specs ship invalid examples
  routinely. Add a per-spec skip list (`bad_examples = ["#/paths/..."]`) to
  that crate's `spec.toml`: every skip is an explicit, commented decision.
  A skipped example that starts passing should flag, so skips can't go stale.
- Rollout: implement in the support crate, run over the existing 60 before
  the +60 onboarding — it will almost certainly catch real bugs immediately,
  on the discord pattern (deserializer and serializer disagreeing on a union).

## 3. T2 — shared assertion helpers (extracted from today's 7 repeated patterns)

The 34 existing tests repeat the same shapes verbatim (audit found:
round-trip skeleton ×9 crates, "unset optional stays off-wire" ×12+ sites,
enum wire-name + Display/FromStr ×3, untagged-union both-variants ×3,
discriminated-union select + reject-unknown ×2). Provide, in the support crate
(see 06):

- `roundtrip!(Type, json!({...}))` — deserialize, re-serialize, idempotence
  check, returns the typed value for follow-on asserts;
- `assert_off_wire!(value, "field_a", "field_b", ...)` — unset optionals do
  not serialize;
- `assert_wire_enum!(Enum, "wire-name" => Enum::Variant, ...)` — wire string
  ↔ variant ↔ Display/FromStr agreement;
- `assert_union_variants!(Union, json_a => Variant_A, json_b => Variant_B)`;
- `assert_rejects!(Type, json!({...}), "reason substring")` — negative cases.

These make a meaningful per-spec suite ~30 lines instead of ~90, which matters
when multiplying across 120 specs.

## 4. T3 — per-spec behavioral minimum (kill constructor-only coverage)

Policy (review discipline, or a support-crate check that counts `#[test]` fns
per spec crate):

every spec promoted from "compiles" to "asserted" gets at least
1 round-trip of a realistic payload, 1 union/enum wire test, and 1 negative
test (unknown discriminant, constraint violation, or both) in its lib.rs test
module. Start by closing the three constructor-only gaps (anthropic,
github-31, openai), then extend over the corpus with priority on specs whose
feature census (02) shows unions/discriminators/constraints.
"All 120 fully asserted" is the direction, not a gate for landing 02 —
T1 gives every spec automatic example coverage on day one.

## 5. T4 — operation-level tests against a mock server

The blind spot of everything above: requests. Nothing today asserts URL
construction, path/query/header encoding, error mapping, or pagination
streaming against a live socket.

- progenitor already has a mock-helper generator: `Generator::httpmock(spec,
  crate_path)` (progenitor-impl/src/httpmock.rs) emits typed
  `httpmock`-based `when/then` helpers per operation. Let
  `conformance_support::generate` optionally emit it as
  `$OUT_DIR/mock.rs`; the spec crate's httpmock + tokio come through the
  support crate's dev face, and the operation tests live in a
  `#[cfg(test)] mod operations` beside the other test modules.
- Per asserted spec, hand-write a handful of operation tests using those
  helpers: one happy-path call asserting the request the client actually sent
  (path encoding via `encode_path`, query explode behavior, header params —
  the (name, location) dedup fix from the review pass deserves a live test),
  one typed 4xx asserting `Error::ErrorResponse` carries the typed body, and
  where applicable one pagination-stream and one multipart/octet-stream body.
- Auto-generating *calls* (synthesizing valid parameter values from schemas)
  is possible — typify's `output_value` machinery can produce instances — but
  is a separate, ambitious follow-up. Scope T4 as: mock helpers wired in +
  hand-written operation tests for the 12 promoted specs + any spec with a
  streaming/upload quirk.

## 6. T5 — compile-time impl assertions

Cheap static guarantees in the spec crates where behavior depends on trait
presence: `fn assert_display<T: std::fmt::Display>() {}` instantiated for
path-parameter types (the Display-degradation fallback from the review pass),
`FromStr` for CLI-relevant types, `Send + Sync` on the client. These turn
"the generator chose the right impl strategy" into a compile failure instead
of a runtime surprise.

## 7. Hygiene fixes bundled here

- Resolve the orphaned `spec-tests/petstore-31.rs`: it becomes a real
  `conformance/petstore-31/` crate (05 §4 step 3; URL verified in 02
  alternates) — a hermetic, tiny, always-asserted spec, and a natural member
  of the PR-CI check subset.
- Re-test the three stale `[lib] doctest = false` flags (figma,
  hetzner-cloud, qdrant) — they predate the fence neutralization. Caveat from
  the audit: figma's comment cites "matrix notation", and if that is
  4-space-indented blocks rather than ``` fences, `neutralize_doc_fences`
  does not cover it; if so, extend the neutralizer (indented-block escaping)
  rather than keeping the flag.
- The copy-paste expect messages in github-31/openai build.rs (both say
  "perplexity") die with the old build.rs bodies in 05 step 2.

## 8. Acceptance criteria

- Example round-trips (T1) run for all corpus specs under
  `cargo test --workspace`, with an explicit per-spec skip ratchet in
  spec.toml; zero unexplained skips.
- No constructor-only asserted specs; the 12 promoted suites all meet the T3
  minimum using T2 helpers.
- At least the 12 promoted specs have T4 operation tests exercising encode,
  typed error, and (where the API offers it) streaming/upload paths.
- A wire-level regression in serializer/deserializer agreement (discord-class
  bug) cannot land without a red test somewhere in the tier.
