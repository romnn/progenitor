// Copyright 2026

//! Smoke-test all conformance specs without invoking rustc.
//!
//! Enumerates every `conformance/*/spec.toml`, fetches the spec (from the
//! `.cache/` directory at the workspace root, downloading on cache miss),
//! parses it, and runs `Generator::generate_text`. Reports per-spec timing
//! and exits non-zero on any failure.
//!
//! Usage:
//!   cargo run -p conformance-support --bin generate-all
//!
//! Set `CONFORMANCE_REFRESH=1` to force re-download of all specs.
//!
//! This is the fast PR-CI gate (~30-60 s for 78 specs, no rustc on generated
//! code) and the local iteration loop when hacking on the generator.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

fn main() {
    let manifests = conformance_support::all_spec_manifests();
    let total = manifests.len();

    // The .cache/ directory lives two levels above CARGO_MANIFEST_DIR:
    // conformance/support/ -> conformance/ -> repo root -> .cache/
    let cache_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("support has parent")
        .parent()
        .expect("conformance has parent (repo root)")
        .join(".cache");
    std::fs::create_dir_all(&cache_root).expect("create .cache/ directory");

    let cache_root = Arc::new(cache_root);

    println!("generate-all: {} specs", total);
    println!("cache: {}", cache_root.display());
    println!();

    let results: Arc<Mutex<Vec<(String, Result<std::time::Duration, String>)>>> =
        Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();
    for (_crate_path, manifest) in manifests {
        let cache_root = Arc::clone(&cache_root);
        let results = Arc::clone(&results);
        let handle = std::thread::spawn(move || {
            let name = manifest.name.clone();
            let t0 = Instant::now();
            let outcome = run_one(&manifest, &cache_root);
            let elapsed = t0.elapsed();
            let result = match outcome {
                Ok(()) => Ok(elapsed),
                Err(e) => Err(e),
            };
            results.lock().unwrap().push((name, result));
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    let mut results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    results.sort_by(|a, b| a.0.cmp(&b.0));

    let mut failures = 0usize;
    for (name, result) in &results {
        match result {
            Ok(elapsed) => {
                println!("  ok  {:>40}  ({:.1}s)", name, elapsed.as_secs_f64());
            }
            Err(e) => {
                failures += 1;
                println!("FAIL  {:>40}", name);
                for line in e.lines() {
                    println!("       {}", line);
                }
            }
        }
    }

    println!();
    println!(
        "{}/{} passed, {} failed",
        total - failures,
        total,
        failures
    );

    if failures > 0 {
        std::process::exit(1);
    }
}

fn run_one(
    manifest: &conformance_support::SpecManifest,
    cache_root: &std::path::Path,
) -> Result<(), String> {
    let document = manifest
        .fetch_document_with_cache(cache_root)
        .map_err(|e| format!("fetch: {e:#}"))?;

    let spec = progenitor_impl::parse_openapi_str(&document)
        .map_err(|e| format!("parse: {e:#}"))?;

    let mut gen = progenitor_impl::Generator::default();
    gen.generate_text(&spec)
        .map_err(|e| format!("generate: {e:#}"))?;

    Ok(())
}
