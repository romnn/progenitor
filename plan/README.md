# Roadmap: corpus scale-out, harness unification, and compile-speed work

Written 2026-06-12, grounded in a measured baseline (commit `9a0cb43`, branch
`feat/openapi3.1-support`): 60/60 wild specs generate, all 60 generated clients
compile, behavioral spec-tests pass, conformance workspace green.

This directory is the plan only — nothing here is implemented yet. Each
workstream has its own document with current state (measured, with file:line
references), design, concrete steps, risks, and acceptance criteria.

## Workstreams

| Doc | Workstream | Maps to request item |
|---|---|---|
| [01-compile-speed.md](01-compile-speed.md) | Make the corpus tiers and generated clients fast to compile | 1 |
| [02-corpus-expansion.md](02-corpus-expansion.md) | Grow corpus 60 → 120 (30 new 3.0.x + 30 new 3.1.x, all verified) | 2 |
| [03-assertions.md](03-assertions.md) | Runtime-correctness assertions for every corpus spec | 3 |
| [04-repo-layout.md](04-repo-layout.md) | Move all crates under `crates/`, flatten `crates/typify/` | 4 (first) |
| [05-harness-unification.md](05-harness-unification.md) | One harness: conformance/ crate-per-spec; remove crates/wild-tests | 4 (second) |
| [06-buildrs-and-deps.md](06-buildrs-and-deps.md) | build.rs simplification, `generate_text` everywhere, dep consolidation | 5 |
| [07-server-generation.md](07-server-generation.md) | Server stub generation (tonic-style traits + axum runtime) — implemented 2026-06-13 | later request |
| [08-impl-architecture.md](08-impl-architecture.md) | Generator-core architecture: correctness fixes, schema-lowering convergence, backend-model extraction, lint enforcement | architecture review 2026-07-10 |

## The one finding that reorders everything

The "32 minutes for a fresh compile tier" is **not rustc being slow on big
crates** — it is the harness running `cargo check -p wild-<name>` for 60
members **one at a time, serially** (`crates/wild-tests/tests/corpus.rs:124-150`).
A valid stripe client (1.0 M lines) cold-checks in **173 s**; incremental in
**41 s**; a no-op re-check costs **0.05 s**. The members are independent leaves
of one shared workspace — checking them in parallel bounds the tier at roughly
the largest crate (cloudflare, ~5-7 min) instead of the sum (~32 min).
Similarly the 206 s tier-1 wall is a serial generation loop. Details and the
rustc pass profile (typeck 35-38 %, borrowck 28 %, coherence 17-18 %, derive
expansion only ~10 %) are in 01.

Roman's call (2026-06-12) on how to get that parallelism: not by fixing the
harness loops but by **deleting the harness** — every corpus spec becomes a
real crate in the `conformance/` workspace (build.rs generation + `include!`),
so cargo's own job scheduler does the parallel type-checking, and real crates
get cargo-expand/rust-analyzer/clippy for free. `crates/wild-tests` is removed
entirely. See 05.

## Sequencing

1. **04 layout move** — one atomic `git mv` commit; everything later lands on
   final paths.
2. **05 conformance conversion** — the big one: support crate, 12 build.rs
   collapsed to 3 lines, the 6 wild spec-test suites become crates,
   `crates/wild-tests` deleted. Delivers the compile-tier parallelism
   structurally (01 Phase A rides along: jobs cap, config niceties).
3. **06 remaining build.rs/dep work** — examples, cargo-progenitor,
   the facade spike (D5).
4. **02 corpus expansion** — scaffold the +60 specs as crates, in batches via
   staging.
5. **03 assertions** — partly parallel with 02; the auto-generated example
   round-trips scale to all 120 specs at once.
6. **01 Phases B–D** (generator code-volume reduction, serde-alternative spike)
   run as a parallel generator-side track; gated on the attribution experiment.

## Decision register

| # | Decision | Where | Status / recommendation |
|---|---|---|---|
| D1 | Move `example-*` crates into `crates/` too, or leave at repo root | 04 | Open. Recommend: move them (clean root; break-list fully known) |
| D2 | Which harness survives | 05 | **Decided by Roman 2026-06-12**: conformance/ crate-per-spec is the one way; `crates/wild-tests` removed entirely (inverts this plan's first draft) |
| D3 | Spec registry shape | 05 | Per-crate `spec.toml` (recommended — the crate is the unit of registration; `generate_all`/scaffolder glob them) vs a central list |
| D4 | sha256 snapshots for cached specs (reproducible corpus) vs trust URLs | 02 | Start without; `spec.toml` has a natural field for it; revisit when a rotating URL first bites CI |
| D5 | Single-runtime-dep facade (generated code resolves serde/reqwest/chrono/… through `progenitor-client` re-exports) | 06 | Run the 1-day spike, then decide; high consumer payoff (luup2), touches every golden |
| D6 | Trim typify's convenience conversion impls (compile-speed lever B2): default-on or opt-in setting | 01 | Opt-in setting first; flip default only if goldens/consumers unaffected |

## Execution discipline (carried over from this session's hard lessons)

- Never write scratch to `/tmp`; use `target/plan-scratch/` or tier scratch dirs
  under `target/`.
- Never redirect cargo/rustc stderr to files; bounded inline diagnostics only
  (`--message-format=short 2>&1 | head -N`). A broken 1 M-line crate once
  produced a 14 GB stderr file; generated code is now prettyplease-formatted
  everywhere precisely to keep diagnostics sane.
- `Generator::generate_text` is the canonical text output; never write
  token-stream `.to_string()` source to disk.
- Corpus `pass` in tier 1 is syn-only; always run the compile tier and
  behavioral tests before claiming correctness.
- Golden regeneration: `EXPECTORATE=overwrite`; goldens stay formatted.
- Run user-given `task ...` commands exactly as given.
