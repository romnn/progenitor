// Copyright 2026

//! Smoke test: parse + generate_text for every spec in the conformance corpus.
//!
//! This is the fast PR-CI gate (~1-2 min at 120 specs): no rustc involved,
//! just fetch → parse → generate. Run with:
//!
//!   cargo test -p conformance-support --test generate_all
//!
//! Cache misses will trigger downloads (or fail if offline). Set
//! `CONFORMANCE_REFRESH=1` to force re-download of all specs.

use conformance_support::all_spec_manifests;

#[test]
fn generate_all_specs() {
    let manifests = all_spec_manifests();
    assert!(!manifests.is_empty(), "no spec.toml files found in conformance/");

    let mut failures = Vec::new();

    for (crate_path, manifest) in &manifests {
        let name = &manifest.name;

        // For file-based specs, read relative to the crate dir.
        let document = if let Some(file) = &manifest.file {
            let path = crate_path.join(file);
            match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(err) => {
                    failures.push(format!("{name}: cannot read {}: {err}", path.display()));
                    continue;
                }
            }
        } else {
            match manifest.fetch_document() {
                Ok(s) => s,
                Err(err) => {
                    failures.push(format!("{name}: fetch failed: {err}"));
                    continue;
                }
            }
        };

        let spec = match progenitor_impl::parse_openapi_str(&document) {
            Ok(s) => s,
            Err(err) => {
                failures.push(format!("{name}: parse failed: {err}"));
                continue;
            }
        };

        if let Err(err) = progenitor_impl::Generator::default().generate_text(&spec) {
            failures.push(format!("{name}: generate failed: {err}"));
        } else {
            println!("  ok  {name}");
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} spec(s) failed generate_all:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
