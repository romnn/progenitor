// Copyright 2026

//! Print the full, untruncated outcome for the named corpus specs
//! (all specs when no names are given). Unlike the ratchet test this
//! exists for debugging: `cargo run -p wild-tests --example diagnose --
//! discord pagerduty`.

use wild_tests::{check_entry, load_manifest};

fn main() {
    let names: Vec<String> = std::env::args().skip(1).collect();
    let specs = load_manifest().expect("manifest loads");

    for entry in &specs {
        if !names.is_empty() && !names.iter().any(|n| n == &entry.name) {
            continue;
        }
        let outcome = check_entry(entry);
        println!("=== {} ({}) ===", entry.name, entry.url);
        println!("expected: {:?}", entry.expect);
        println!("outcome:  {}", outcome.label());
        if let Some(detail) = outcome.detail() {
            println!("detail:\n{detail}");
        }
        println!();
    }
}
