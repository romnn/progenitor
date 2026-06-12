# 01 — Compile-speed: measured baseline and the plan

Request: "206 s for check and 32 min for a fresh build is very slow! … rust
should be able to compile faster if it does things in parallel on this 24-core
machine! maybe more smaller files? or maybe going for facet over serde? …
maybe parallel rust frontend on nightly, maybe more codegen units."

We profiled before planning. Several of the intuitive levers turn out to be
measured dead ends, and the two real wins are (a) the harness runs everything
serially today and (b) typify emits an enormous *impl* surface that dominates
the rustc frontend. All numbers from 2026-06-12 on this machine (Threadripper
9960X, 24c/48t, 60 GB RAM, stable rustc 1.96.0, nightly 1.98.0).

## 1. What the two headline numbers actually are

- **206 s** is tier 1 (`corpus_matches_manifest` with `WILD_SLOW=1`): fetch
  (cached) → parse → `generate_tokens` → `syn::parse2`, for 60 specs, in a
  **serial `for` loop** (`crates/wild-tests/tests/corpus.rs:32-66`). This is
  *our* generator's wall time, single-threaded. No rustc involved.
- **1919 s (~32 min)** is tier 2 (`compile_generated_clients`): one shared
  throwaway workspace, then `cargo check/test -p wild-<name>` **per member, one
  at a time, `.status()`-waited** (`corpus.rs:124-150`). Dependencies compile
  once (~13 s, parallel); the 60 member frontends run strictly sequentially.

## 2. Measured facts

Per-crate wall (fresh target dir, `cargo check -p`, stable):

| | stripe (1.01 M lines, 37.9 MB) | github-31 (588 k lines, 20.9 MB) |
|---|---|---|
| cold check | **173 s** | **121 s** |
| incremental after `touch src/lib.rs` | 41 s | 22 s |
| no-op re-check | 0.05 s | 0.10 s |
| peak RSS | 10.2 GB | 6.0 GB |

rustc pass breakdown (`-Ztime-passes`, share of frontend total):

| pass | stripe | github-31 |
|---|---|---|
| type checking | 35 % | 38 % |
| MIR borrow checking | 28 % | 28 % |
| **coherence checking** | **17 %** | **18 %** |
| macro/derive expansion | 10 % | 11 % |
| name resolution | 4 % | 3 % |
| crate metadata | 3 % | 3 % |
| lints | 1.5 % | 1.3 % |
| parsing | **< 1 %** | < 1 % |

Why coherence is a startling #3: the generated code is impl-saturated. Census
of the largest clients (prettyplease-formatted sources from the current tier):

| crate | lines | structs | enums | impl blocks | of which TryFrom / FromStr / From / Display |
|---|---|---|---|---|---|
| cloudflare | 2 459 510 | 16 112 | 7 281 | 60 605 | 25 947 / 9 403 / 7 416 / 7 306 |
| chargebee | 1 248 911 | 9 112 | 3 223 | 59 472 | 27 063 / 9 021 / 6 501 / 2 529 |
| stripe | 1 010 257 | 8 122 | 3 244 | 52 242 | 23 004 / 7 668 / 6 925 / 2 443 |

≈ 4.5-6 impl blocks per type; **conversion impls are ~75 % of all impls**.
These come from typify's unconditional per-enum family
(Display/FromStr/TryFrom<&str>/TryFrom<&String>/TryFrom<String>, plus Default —
`crates/typify/typify-impl/src/type_entry.rs:1010-1062`) and per-newtype
Deref/From/TryFrom scaffolding (`type_entry.rs:1078-1133`).

Two more measured facts that shape the plan:

- **Broken generations are pathological**: a pre-fix stripe (syn-valid but
  type-broken) ran **> 792 s without finishing** (killed; 2013 E0599 errors
  emitted, suggestion machinery is the cost). Valid stripe: 173 s. The
  folklore comment "Stripe alone costs rustc the better part of an hour"
  (`corpus.rs:98-99`) almost certainly dates from broken output and should be
  corrected.
- **Memory bounds parallelism**: ~10 GB RSS for stripe's frontend; cloudflare
  will be higher. 60 GB RAM supports ~4-5 concurrent monster frontends.

## 3. Measured dead ends (answering the request's hypotheses directly)

| Hypothesis | Verdict |
|---|---|
| "more smaller files" | Parsing is < 1 % of frontend time, and the stable frontend is single-threaded per crate regardless of file count. Splitting the output into modules/files is a developer-experience improvement (editors, diffs), not a compile-speed lever. Splitting into multiple *crates* would help — see Phase B3 note — but is incompatible with the `include!`-based build.rs consumer mode. |
| "more codegen units" | `cargo check` does no codegen at all, and dev builds already default to 256 CGUs. Confirmed irrelevant: full `cargo build` of a mid-size client with `debug=0` vs `debug=2` differs by **~2 %** — even unoptimized *builds* of these crates are frontend-bound. |
| "parallel rust frontend on nightly" | Benchmarked earlier this week: ~1.5× cold-check win, ~3× *regression* on incremental, nightly-only. Rejected as a default (recorded in `conformance/Cargo.toml` header). Not worth re-litigating until the feature stabilizes. |
| "facet over serde" | Derive expansion is only ~10 % of frontend time. The *expanded* serde code does also feed typeck/borrowck, so the true serde-attributable share is larger than 10 % — but unknown. Phase B1 measures it before we invest; Phase D defines the spike and its kill criteria. |

## 4. Phase A — parallelism (delivered structurally by workstream 05)

The first draft of this plan fixed the serial loops *inside* the harness
(workspace-wide `cargo check`, rayon over the generation loop,
write-if-changed). Roman's call on 05 makes that harness code unnecessary —
the corpus becomes real member crates of the `conformance/` workspace, and
**cargo itself is the parallel scheduler**:

- the 60-serial-checks problem disappears: `cargo check --workspace` runs
  member frontends in parallel; wall ≈ the largest member (cloudflare,
  ~5-7 min estimated from the byte→time scaling) instead of the ~32-min sum;
- the serial generation loop disappears: generation runs in each crate's
  build.rs, and cargo runs build scripts in parallel too;
- write-if-changed is subsumed by cargo fingerprinting: untouched specs are
  no-ops; editing the generator (a build-dependency) correctly regenerates
  and re-checks everything — in parallel;
- the fast no-rustc gate (old tier 1) survives as the support crate's
  `generate_all` smoke test (bounded thread pool over all spec.tomls,
  fetch → parse → `generate_text`; expected ~30-50 s for 60 specs, ~1-2 min
  at 120).

What Phase A still owns (small, lands with 05):

- **A1. Memory cap**: `build.jobs` ≈ 10-12 in `conformance/.cargo/config.toml`
  — monsters peak 6-15 GB per frontend; an uncapped `-j48` schedule can
  co-run enough of them to exhaust 60 GB.
- **A2. Timing visibility**: `cargo check --timings` already gives per-unit
  wall charts (used for the measurements above); the `generate_all` smoke
  prints per-spec generation ms. No custom instrumentation needed.
- **A3. Broken-monster caveat, documented**: a generation that is syn-valid
  but type-broken still hits rustc's pathological error path (>13 min, §2).
  There is no per-unit cargo timeout; the protection is the fix-forward
  membership policy (a red crate never merges) plus staging for
  works-in-progress (05 §1). Correct the stale "better part of an hour"
  comment wherever it migrates.

Acceptance: fresh conformance `cargo check --workspace` ≤ ~8 min wall locally;
unchanged re-check ≤ ~1 min; `generate_all` smoke ≤ 60 s at 60 specs; peak RSS
under the jobs cap < 50 GB.

## 5. Phase B — shrink what we generate (weeks; attacks the 80 % frontend share)

Typeck + borrowck + coherence = ~80 % of frontend time, all three scaling with
the volume of emitted items. We own typify now; this is the structural lever.

- **B1. Attribution experiment (do first, ~1 day).** Generate stripe twice
  with quick hacked typify branches: (a) conversion-impl family suppressed,
  (b) serde derives suppressed (types only, won't compile as a client — check
  the types module alone). Compare cold-check times against baseline. This
  splits the 80 % between "typify's explicit impls" and "serde-derive-expanded
  impls" and decides whether Phase D (serde alternatives) is worth anything.
- **B2. Impl-profile setting in typify.** New `TypeSpaceSettings` knob (e.g.
  `with_impl_profile(Full | Slim)`), surfaced through `GenerationSettings`.
  Candidates to gate behind `Full` (audit before cutting — some are
  load-bearing):
  - keep: `Display`/`FromStr` (progenitor uses `has_impl(Display)` for path
    params and `FromStr` for CLI parsing), the bespoke
    `impl Deserialize` for constrained types and `TryFrom` guards (these are
    the wire-correctness machinery from the discord/clickhouse fixes), serde
    derives;
  - candidates to drop in `Slim`: the delegating duplicate conversions
    `TryFrom<&String>` / `TryFrom<String>` (keep one canonical `TryFrom<&str>`),
    `From<T>`-for-String conveniences, `Deref` on unconstrained newtypes,
    `Default` on enums where unused, `ToString`-adjacent duplication.
  - Estimate: cutting 2-3 impls per enum/newtype removes ~30-40 % of impl
    blocks on monsters; coherence (17 %) shrinks superlinearly with impl
    count, typeck/borrowck linearly with the removed bodies. Honest overall
    guess: 20-35 % cold-check win on monsters. B1 firms this up.
  - Rollout: opt-in setting first; corpus tier adopts it; flip the default
    only after confirming no golden/consumer (luup2, oxide-style specs)
    depends on the dropped impls (decision D6).
- **B3. Structural type dedup (investigation).** cloudflare emits 16 k structs;
  wild specs repeat shapes heavily (per-operation error envelopes etc.).
  Instrument typify finalize to hash structural shapes and report duplicate
  groups per corpus spec. If the duplication factor is large (> 1.5×), design
  aliasing (one named type + `pub type` aliases) as a follow-up. Unknown
  payoff; one-day instrumentation before committing.
- **B4. (Noted, not planned) multi-crate sharding.** typify knows the type
  DAG; sharding a monster's types into N crates along SCC-condensation layers
  would parallelize the *frontend* on stable rustc. Big design (cross-crate
  visibility, re-export facade), and useless for `include!`-mode consumers.
  Revisit only if B1-B3 land and monsters are still the critical path.

## 6. Phase C — toolchain settings (mostly "documented no")

- Keep stable as the default toolchain; re-evaluate `-Zthreads` only when the
  parallel frontend stabilizes.
- Optional nicety: `conformance/.cargo/config.toml` enabling a faster linker
  (mold/lld) for the test binaries, and `[profile.dev] debug = 0` (worth ~2 %
  per the experiment, but free). Do it with 05's config file.
- Skip: codegen-units tuning, sccache (check artifacts aren't cacheable by
  sccache; CI caching is better served by actions/cache on the conformance
  target dir keyed on generator-source + spec-cache hashes, see 05 §3).

## 7. Phase D — serde-alternative backend (gated spike, after B1)

Only if B1 attributes a meaningful share (> 25 % of frontend) to
derive-expanded serde code:

- Disqualified up front: **miniserde** (no untagged enums, no
  internally-tagged enums, no flatten — our output depends on untagged
  heavily), **nanoserde** (same class of gaps).
- **facet** is the only credible candidate (derive emits a shape descriptor;
  serialization is runtime reflection → much less expanded code). Spike
  checklist before any integration work:
  - representation parity: untagged unions, internally/adjacently tagged
    enums, `rename`/`rename_all`, `default`, `skip_serializing_if`-equivalent
    omission of unset optionals, transparent newtypes, map keys;
  - hook parity for our *custom* constrained-type deserialization (pattern /
    range / enum-membership guards currently live in hand-emitted
    `impl Deserialize` blocks — facet needs an equivalent validation hook);
  - client integration: `reqwest::Response::json()` is serde-bound; the
    generated client and `progenitor-client::ResponseValue` would need a
    facet code path (`.bytes()` + `facet_json::from_slice`), implying a
    progenitor-client feature or variant;
  - maturity: facet is pre-1.0 and API-churning; pin exactly, vendor if needed.
- Design shape if it passes: a **generation-time option**
  (`GenerationSettings::serde_backend = Serde | Facet`), not cfg-features
  inside generated code — the user picks at generation time, which delivers
  the requested "users can choose between faster runtime (serde) or faster
  compile time" without doubling every emitted item behind `cfg_attr`.
- Kill criteria: < 20 % measured check-time win on stripe, or any wire-format
  divergence in the conformance/spec-test suites (byte-level round-trip
  parity required), or unmaintainable validation-hook hacks.

## 8. Order of work

Phase A rides along with workstream 05 (the structural move is the
parallelism fix). Then B1 (attribution experiment), then B2 behind a setting,
B3 instrumentation in parallel; D only per its gate; the linker/profile
niceties (C) land in conformance/.cargo/config.toml with 05. Re-measure after
each landing (`cargo check --timings`, `generate_all` smoke output) and record
numbers in the commit message.
