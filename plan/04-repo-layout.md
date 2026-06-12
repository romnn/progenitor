# 04 — Repo layout: everything under `crates/`, typify flattened

Request: "move all progenitor-* crates into crates/ and flatten crates/typify/
into crates/ so we have crates/progenitor-impl, crates/typify-impl etc."

## 1. Target tree

```
crates/
  progenitor/            (facade; was ./progenitor)
  progenitor-impl/
  progenitor-macro/
  progenitor-client/
  cargo-progenitor/
  typify/                (was crates/typify/typify)
  typify-impl/           (was crates/typify/typify-impl)
  typify-macro/          (was crates/typify/typify-macro)
  wild-tests/            (unchanged here; removed entirely by 05 — its
                          fetch/cache logic moves to conformance/support)
  example-build/         (D1: recommended to move; see below)
  example-macro/
  example-out-dir/
  example-wasm/
sample_openapi/          (stays at root — 20+ references, all fixable by one rule)
docs/  conformance/  plan/  README.md  CHANGELOG.adoc  Cargo.toml  ...
```

`crates/typify` today is a plain directory (no Cargo.toml of its own) holding
the three crates plus three loose files: `LICENSE`, `README.md`,
`example.json`. Flattening means relocating those too (§3).

Decision D1: the `example-*` crates. Recommendation: move them into `crates/`
with everything else — the motive for this workstream is a clean root, and
their entire break-list is known (4 `../sample_openapi` strings, their
path-deps survive if they move together, one CI `cd example-wasm`). If kept at
root instead, their `progenitor*` path deps must change to `../crates/...` —
either way is a small, known edit set.

## 2. Complete break inventory (from a full-repo audit; nothing else found)

Workspace and manifests:

- Root `Cargo.toml`: members list (lines 3-15) and `[workspace.dependencies]`
  path entries for progenitor/-client/-impl/-macro (26-29) and
  typify/typify-impl/typify-macro (81-83). Also delete the stale commented
  `[patch.crates-io]` blocks (88-94) referencing `../typify` — inert and
  misleading after the move.
- `readme = "../README.md"` in **seven** crate manifests resolves one level
  too shallow after the move → `../../README.md`: progenitor, progenitor-impl,
  progenitor-macro, cargo-progenitor (each `Cargo.toml:9`), and typify,
  typify-impl, typify-macro (each `Cargo.toml:8`, which today point at
  `crates/typify/README.md` — see §3 for where that file goes).
- `conformance/Cargo.toml:31-32`: `path = "../progenitor"` →
  `"../crates/progenitor"`, same for progenitor-client. (`wild-tests` path
  already `../crates/wild-tests`, unchanged.)
- Survive untouched (verified, easy to second-guess — don't):
  progenitor-macro → progenitor-impl (`../progenitor-impl`, both move),
  typify-macro → typify-impl (`../typify-impl`, siblings after flatten),
  `progenitor-impl/tests/output/Cargo.toml:17`
  (`../../../progenitor-client` — three-up lands at the crates/ dir where
  progenitor-client remains a sibling), all expectorate golden paths
  (crate-relative), conformance member manifests (workspace-dep only).

Source code:

- `crates/wild-tests/src/lib.rs:346`:
  `include_str!("../../../progenitor-client/src/progenitor_client.rs")` →
  `"../../progenitor-client/src/progenitor_client.rs"` (wild-tests stays put
  while progenitor-client becomes its sibling under `crates/`, so the include
  goes from three-up to two-up). Transitional: 05 deletes this file along
  with the crate, but 04 lands first and must leave everything green.
- `../sample_openapi/...` references, all one level deeper if their crate
  moves: example-build/build.rs:6, example-out-dir/build.rs:11,
  example-wasm/build.rs:6, example-macro/src/main.rs:6 and :35 (macro paths
  resolve against CARGO_MANIFEST_DIR), progenitor-impl/tests/test_output.rs:37
  and :161, and the five progenitor/tests/build_*.rs + load_yaml.rs files →
  all become `../../sample_openapi/...`.

typify's `example.json` (the subtle one):

- Nine **live doctests** invoke `import_types!("../example.json")` resolved
  against the invoking crate's manifest dir: 8 occurrences in
  `crates/typify/typify/src/lib.rs`, 1 in `typify-macro/src/lib.rs:27`.
  After flattening, `../example.json` would resolve to `crates/example.json`.

Meta files:

- `.gitignore:3` `progenitor-impl/tests/output/Cargo.lock` → prefix `crates/`.
- `.tokeignore`: lines 7 (progenitor-impl tests output), 14-16 (typify
  schemas/goldens paths), and 10 (`sample_openapi/` — only if it moved; it
  doesn't).
- `.github/workflows/rust.yml:82` `cd example-wasm` → `cd crates/example-wasm`
  (the ONLY path-coupled CI step; everything else is `--locked` from root with
  no `-p` flags — the workflow is otherwise layout-agnostic).
- `README.md:115,366` build.rs snippets showing `../sample_openapi/keeper.json`
  mirror the old depth — update the docs to match whatever consumers should
  write (they're illustrative; consumer repos have their own layout, so
  consider rewriting the snippet to a neutral `"openapi/keeper.json"` style
  while touching it).

Release tooling:

- Root `release.toml` pre-release-replacements use `file = "../CHANGELOG.adoc"`,
  resolved relative to each releasing package root. Today that is correct only
  for depth-1 crates and is **already latently broken** for the nested typify
  crates that inherit it. After the move every releasable crate sits at depth
  2, so a single `../../CHANGELOG.adoc` works uniformly — the move actually
  *fixes* the inconsistency. Update the three replacement entries.

## 3. The loose-file decisions inside `crates/typify/`

- `example.json`: move into `crates/typify/` (the wrapper crate, i.e. the old
  `crates/typify/typify/`) and update the doctests: the 8 typify occurrences
  become `import_types!("example.json")`; the 1 typify-macro occurrence
  becomes `"../typify/example.json"`. This keeps the file owned by the crate
  whose docs use it instead of polluting `crates/`.
- `crates/typify/README.md`: becomes the typify wrapper crate's README
  (`crates/typify/README.md` post-move); typify-impl/typify-macro `readme`
  fields point at it (`../typify/README.md`).
- `crates/typify/LICENSE` (Apache-2.0, distinct from the repo's MPL-2.0):
  must stay adjacent to the typify crates for packaging. Place a copy in each
  of the three typify crates (`license = "Apache-2.0"` SPDX field means cargo
  wants the file shipped; cargo auto-includes a crate-root LICENSE) — or keep
  one at `crates/typify/LICENSE` and add `license-file` is *not* needed with
  SPDX; verify `cargo package --list` output for the three crates as part of
  the gate.

## 4. Execution

One atomic commit, pure mechanics, zero code changes beyond path strings:

1. `git mv` each crate (preserves history): the five progenitor crates +
   cargo-progenitor + the four examples (per D1) into `crates/`; `git mv
   crates/typify/typify crates/typify-tmp && git mv crates/typify/typify-impl
   crates/typify-impl && ...` then the loose files, then rename typify-tmp →
   typify (two-step because the parent dir is being dissolved).
2. Apply every edit in §2/§3 — the inventory above is intended to be complete;
   work through it as a checklist.
3. Regenerate both lockfiles (`cargo update --workspace` is NOT needed —
   path-dep moves don't change lock entries; but run `cargo metadata` to
   confirm both workspaces resolve, then `cargo build --locked` to prove CI's
   `--locked` stays green).
4. Gate: full root-workspace test run + conformance workspace test run +
   tier 1 corpus + `cargo package --list -p typify -p typify-impl -p
   typify-macro -p progenitor -p progenitor-impl -p progenitor-macro -p
   progenitor-client` (readme/license paths resolve) + grep for any leftover
   `"../progenitor` / `"progenitor-impl` / `crates/typify/typify` strings.

## 5. Risks

- Lowest-risk workstream of the six; the only subtle items are the typify
  doctest paths (they are *live* doctests — they run `import_types!` and will
  fail loudly if wrong) and the `cargo package` file-resolution fields, both
  covered by the gate.
- Do it before 05/06 so those workstreams land on final paths, and keep it
  free of opportunistic refactors — any code change belongs to another
  workstream.
