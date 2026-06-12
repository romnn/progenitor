# 05 — One harness: conformance/ crate-per-spec; remove crates/wild-tests

Decided by Roman 2026-06-12 (inverting this plan's first draft, which kept the
wild-tests engine and deleted conformance/): **the conformance model wins**.
One real crate per spec in the `conformance/` workspace; `crates/wild-tests`
is removed entirely. Rationale:

- Workspace members **type-check in parallel natively** — cargo is the
  scheduler. The serial `corpus.rs` loops (the measured cause of the 32-min
  tier, see 01 §1) are not fixed but *deleted*; there is no harness loop left
  to get wrong.
- Real crates mean real tooling per spec: `cargo expand -p
  conformance-discord`, rust-analyzer over the `include!`d codegen,
  `cargo clippy -p`, `cargo doc -p`, and a browsable prettyplease-formatted
  `codegen.rs` in OUT_DIR.
- One management surface: a spec **is** a crate; its tests live in its own
  `src/lib.rs`; adding/removing a spec is adding/removing a crate.

## 1. Capability mapping: what wild-tests provided → its replacement

| wild-tests capability | replacement in the crate-per-spec model |
|---|---|
| `manifest.toml` central registry | per-crate `spec.toml` next to each Cargo.toml (`name`, `url =` or `file =`, optional `sha256` — fits D4 — and quirk notes such as the Stainless URL-rotation recipe). Machine-readable inventory = `conformance/*/spec.toml` glob; duplicate protection = directory names. |
| `fetch`/cache + `WILD_REFRESH` | moves into `conformance/support` (the 06 support crate). Shared cache at `conformance/support/cache/` (gitignored, same mechanism, same ~152 MB moves over); `CONFORMANCE_REFRESH=1` to re-download. |
| tier 1 (fast generate+syn gate, no rustc on generated code) | a `generate_all` smoke test in the support crate: enumerate sibling `spec.toml`s, fetch → parse → `generate_text` in a bounded thread pool, report per-spec timing. This is the cheap PR-CI gate (~1-2 min at 120 specs) and the fast local loop for generator hacking. |
| tier 2 (compile generated clients) | plain `cargo check --workspace` in conformance/ — parallel, wall ≈ the largest member (cloudflare, ~5-7 min locally). `cargo test --workspace` additionally builds each lib's test binary (codegen) — inherent once every crate has tests (03/T3); the check/test distinction is the only "tier" left. |
| `expect = pass/parse-fail/…` outcome ratchet | **workspace membership is the ratchet.** All 60 specs pass today and we own the whole stack — the policy is fix-forward, the members list never contains a red crate. A spec we cannot make pass yet lives as a scaffolded-but-unlisted directory (`conformance/staging/<name>/` or a commented member with a tracking note) until promoted. |
| `slow` flags + `WILD_SLOW`/`WILD_COMPILE*` env vars | gone. Locally everything is just a member; single-spec iteration is `cargo check -p conformance-<name>` — better DX than env-var gating. Memory is the only reason to throttle: monsters are 6-15 GB frontends, so set `build.jobs` (~10-12 on this 60 GB machine) in `conformance/.cargo/config.toml`. |
| `examples/diagnose` (untruncated outcome) | `cargo check -p conformance-<name>`: a generation failure fails build.rs with the full error; a type-level failure is rustc's own diagnosis. |
| `examples/emit` (materialize for inspection) | already materialized: `target/debug/build/conformance-<name>-*/out/codegen.rs` (prettyplease-formatted); `cargo expand -p` for the post-derive view. The support crate can grow a `--bin where` helper that prints the path if the glob gets old. |
| `write_compile_crate` + embedded `progenitor_client.rs` module + forced `doctest = false` | gone. Every crate uses the **real progenitor-client path dep** — consumer fidelity everywhere, and **doctests on generated code default ON** across the whole corpus (the embedded-module reason for disabling them disappears; keep per-crate opt-out for pathological specs, re-test the 3 legacy flags per 03 §7). |
| per-spec behavioral suites in `spec-tests/<name>.rs` | inline `#[cfg(test)] mod tests` in `conformance/<name>/src/lib.rs`, exactly like the existing 12 crates (the user-named target). The 6 wild suites migrate; discord's two complementary suites merge. |

Net deletions: `corpus.rs`, `write_compile_workspace`/`write_compile_crate`,
the throwaway workspace, the embedded-client hack, the orphaned
`petstore-31.rs` (becomes a real tiny crate), the whole
`crates/wild-tests` directory. Preserve in support-crate docs: the
14 GB-single-line-stderr lesson and the parallel-frontend benchmark verdict
currently recorded in comments there and in conformance/Cargo.toml's header.

## 2. Anatomy of one spec crate (the only pattern in the repo)

```
conformance/discord/
  Cargo.toml      # ~12 lines: [package]; [dependencies] progenitor-client + runtime
                  # one-liners ({ workspace = true }); [build-dependencies]
                  # conformance-support; [dev-dependencies] conformance-support
  spec.toml       # name = "discord"
                  # url = "https://raw.githubusercontent.com/discord/..."
                  # notes = "oneOf/discriminator + null types; inline enum discriminants"
  build.rs        # fn main() { conformance_support::generate("spec.toml") }
  src/lib.rs      # doc comment; include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
                  # #[cfg(test)] mod tests { ... behavioral assertions ... }
```

Nothing implicit: build.rs names the exact file that drives generation (cargo
runs build scripts with cwd = the crate root, so `"spec.toml"` is plain
relative-path fs, no env-var tricks), and spec.toml states the spec's name and
source in full. `generate` asserts `name` matches the crate directory, which
makes the copy-paste-name bug class (two of the existing 12 build.rs carry a
wrong spec name today) a hard error instead of a latent typo. Hermetic crates
(petstore-31, keeper-based) say `file = "petstore-31.json"` instead of `url =`
— the source kind is explicit in the toml, one `generate` either way.

Scaffolder so 120 crates stay cheap to mint:
`cargo run -p conformance-support --bin new-spec -- <name> <url>` writes the
four files. Cargo-native behaviors we get for free and should document rather
than re-implement: build.rs reruns when the cached spec, spec.toml, or any
build-dependency (i.e. the generator itself) changes — editing progenitor-impl
correctly regenerates and re-checks everything, in parallel; untouched specs
are fingerprint-clean no-ops.

## 3. CI story (the one real cost of this model — be honest about it)

A full `cargo check --workspace` over 120 members does not fit standard GitHub
runners (4 cores/16 GB; stripe alone peaks 10.2 GB). Plan:

- **PR CI**: the `generate_all` smoke (cheap, no rustc on generated code,
  network-or-cache) + `cargo check -p` for a pinned small subset (petstore-31
  and ~5 small spec crates) + the root workspace as today.
- **Nightly**: full conformance `cargo check --workspace` + `cargo test
  --workspace` on a larger runner (or accept a long wall on the standard one
  with `build.jobs = 2-3`; measure once and pick).
- Cache `conformance/support/cache/` (specs) and the conformance target dir
  keyed on generator-source + spec-cache hashes.

Macro-mode and prebuilt-crate consumer coverage (the old "consumer-modes"
idea) shrinks: build.rs+`include!` fidelity is now what *every* conformance
crate exercises; `example-macro` (root workspace) keeps the macro path
covered; add a cargo-progenitor prebuilt smoke under 06. No separate
consumer-modes workspace needed.

## 4. Migration steps (after 04's layout move)

1. Create `conformance/support/` (fetch/cache/`generate("spec.toml")`/
   spec.toml parsing/`generate_all` smoke/`new-spec` scaffolder — design in
   06); move the cache directory contents over.
2. Convert the 12 existing crates: add `spec.toml`, shrink build.rs to the
   3-line form (drops `wild-tests`, `syn`, `prettyplease`, and discord's dead
   `serde_json` from `[build-dependencies]`; the github-31/openai expect-message
   typos die with the old build.rs bodies).
3. Scaffold crates for the 6 wild spec-test specs (clickhouse-cloud,
   cloudflare, codat-accounting, mongodb-atlas, pagerduty; discord exists) and
   port `spec-tests/*.rs` into their `src/lib.rs` test modules; merge the two
   discord suites; add `petstore-31` as a real tiny crate (03 §7).
4. Scaffold the remaining ~42 corpus specs as crates, in batches, keeping
   `cargo check --workspace` green per batch (manifest.toml is the source for
   names/URLs/notes until it is fully transcribed, then deleted).
5. Delete `crates/wild-tests`: root workspace member entry, `.tokeignore`
   cache line, the `WILD_*` env-var documentation; update README/docs/memory
   ("wild corpus" → "conformance corpus").
6. Wire CI per §3; add `conformance/.cargo/config.toml` (`build.jobs` cap,
   optionally `[profile.dev] debug = 0` and a mold/lld linker stanza — cheap,
   per 01 Phase C).

## 5. Acceptance criteria

- `crates/wild-tests` no longer exists; `conformance/` holds support + one
  crate per corpus spec (61 + support before expansion; 121 after 02), all
  green under `cargo check --workspace` and `cargo test --workspace`.
- No serial harness code anywhere: parallelism is cargo's job scheduling,
  bounded only by `build.jobs`.
- All 34 + 53 existing assertions survive the migration (run before/after
  inventories); doctests run on generated code corpus-wide.
- Adding a spec = `new-spec` scaffold + fill in tests; no env vars in any
  workflow; `cargo expand -p conformance-<name>` works for every spec.
