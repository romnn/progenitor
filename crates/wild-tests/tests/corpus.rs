// Copyright 2026

//! Corpus ratchet: every manifest entry must produce exactly its expected
//! outcome. A regression (pass → fail) and an unrecorded improvement
//! (fail → pass) both fail the suite — update `manifest.toml` to lock in
//! changes deliberately.
//!
//! Environment knobs:
//! - `WILD_SLOW=1` — include entries marked `slow` (multi-minute specs).
//! - `WILD_REFRESH=1` — re-download cached documents.
//! - `WILD_COMPILE=1` — additionally `cargo check` the generated client
//!   of every entry expected to pass (tier 2; slow).

use wild_tests::{Expectation, Outcome, check_entry, load_manifest, write_compile_crate};

#[test]
fn corpus_matches_manifest() {
    let specs = load_manifest().expect("manifest loads");
    assert!(!specs.is_empty(), "manifest must not be empty");

    let run_slow = std::env::var_os("WILD_SLOW").is_some_and(|v| v == "1");

    let mut failures = Vec::new();
    let mut unavailable = Vec::new();
    let mut skipped = 0usize;

    for entry in &specs {
        if entry.slow && !run_slow {
            skipped += 1;
            continue;
        }
        let outcome = check_entry(entry);
        let status = if outcome.matches(entry.expect) {
            let detail = outcome.detail().unwrap_or_default();
            let detail: String = detail.chars().take(120).collect();
            if detail.is_empty() {
                "ok".to_string()
            } else {
                format!("ok — {detail}")
            }
        } else if matches!(outcome, Outcome::Unavailable(_)) {
            // Network problems are not verdicts; report but don't fail.
            unavailable.push(format!(
                "{}: {}",
                entry.name,
                outcome.detail().unwrap_or_default()
            ));
            continue;
        } else {
            failures.push(format!(
                "{}: expected {:?}, got {}",
                entry.name,
                entry.expect,
                outcome.label(),
            ));
            let detail = outcome.detail().unwrap_or_default();
            let detail: String = detail.chars().take(160).collect();
            format!("MISMATCH — {detail}")
        };
        println!("{:<28} {:<14} {}", entry.name, outcome.label(), status);
    }

    if !unavailable.is_empty() {
        println!("\nunavailable (not counted):");
        for line in &unavailable {
            println!("  {line}");
        }
    }
    if skipped > 0 {
        println!("\n{skipped} slow specs skipped (set WILD_SLOW=1 to include)");
    }

    assert!(
        failures.is_empty(),
        "corpus outcomes diverged from manifest:\n{}",
        failures.join("\n")
    );
}

#[test]
fn compile_generated_clients() {
    if std::env::var_os("WILD_COMPILE").is_none_or(|v| v != "1") {
        eprintln!("set WILD_COMPILE=1 to compile-check generated clients");
        return;
    }
    let run_slow = std::env::var_os("WILD_SLOW").is_some_and(|v| v == "1");
    let out_root = std::env::temp_dir().join("progenitor-wild-compile");

    let mut failures = Vec::new();
    for entry in load_manifest().expect("manifest loads") {
        if entry.expect != Expectation::Pass || (entry.slow && !run_slow) {
            continue;
        }
        let crate_dir = match write_compile_crate(&entry, &out_root) {
            Ok(dir) => dir,
            Err(err) => {
                failures.push(format!("{}: generation failed: {err}", entry.name));
                continue;
            }
        };
        let status = std::process::Command::new("cargo")
            .arg("check")
            .arg("--quiet")
            .current_dir(&crate_dir)
            .status()
            .expect("cargo runs");
        println!(
            "{:<28} {}",
            entry.name,
            if status.success() {
                "compiles"
            } else {
                "FAILS"
            }
        );
        if !status.success() {
            failures.push(format!("{}: generated client does not compile", entry.name));
        }
    }
    assert!(
        failures.is_empty(),
        "generated clients failed to compile:\n{}",
        failures.join("\n")
    );
}
