// Copyright 2026

//! Scaffold a new conformance spec crate.
//!
//! Usage:
//!   cargo run -p conformance-support --bin new-spec -- <name> <url>
//!
//! Creates `conformance/<name>/` with Cargo.toml, spec.toml, build.rs, and
//! src/lib.rs. Does not add the crate to the conformance workspace — do that
//! manually after verifying the spec generates cleanly.
//!
//! To verify before promoting:
//!   cargo check -p conformance-<name>

use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: new-spec <name> <url>");
        eprintln!("  name  crate name (becomes conformance/<name>/ and package conformance-<name>)");
        eprintln!("  url   URL to the OpenAPI spec (yaml or json)");
        std::process::exit(1);
    }

    let name = &args[1];
    let url = &args[2];

    // Validate name: lowercase, hyphens only.
    if !name.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit()) {
        eprintln!("error: name must be lowercase with hyphens only, got {:?}", name);
        std::process::exit(1);
    }

    // Find the conformance root: two levels up from this binary's CARGO_MANIFEST_DIR.
    let support_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let conformance_root = support_dir.parent().expect("support has parent");
    let crate_dir = conformance_root.join(name);

    if crate_dir.exists() {
        eprintln!("error: {} already exists", crate_dir.display());
        std::process::exit(1);
    }

    std::fs::create_dir_all(crate_dir.join("src")).expect("create src/");

    // spec.toml
    std::fs::write(
        crate_dir.join("spec.toml"),
        format!(
            r#"name = "{name}"
url = "{url}"
version = ""
notes = ""
"#
        ),
    )
    .expect("write spec.toml");

    // Cargo.toml
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "conformance-{name}"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
progenitor-client = {{ workspace = true }}
bytes = {{ workspace = true }}
chrono = {{ workspace = true }}
futures-core = {{ workspace = true }}
percent-encoding = {{ workspace = true }}
regress = {{ workspace = true }}
reqwest = {{ workspace = true }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
serde_urlencoded = {{ workspace = true }}
uuid = {{ workspace = true }}

[build-dependencies]
conformance-support = {{ workspace = true }}

[dev-dependencies]
conformance-support = {{ workspace = true }}
"#
        ),
    )
    .expect("write Cargo.toml");

    // build.rs
    std::fs::write(
        crate_dir.join("build.rs"),
        r#"fn main() {
    conformance_support::generate("spec.toml");
}
"#,
    )
    .expect("write build.rs");

    // src/lib.rs
    std::fs::write(
        crate_dir.join("src/lib.rs"),
        format!(
            r#"//! Generated {name} API client — conformance crate.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn client_constructs() {{
        // Replace with the correct base URL for this API.
        let _client = Client::new("https://api.example.com");
    }}
}}
"#
        ),
    )
    .expect("write src/lib.rs");

    println!("Created {}", crate_dir.display());
    println!();
    println!("Next steps:");
    println!("  1. Add {:?} to conformance/Cargo.toml members (or staging/).", name);
    println!("  2. Fill in spec.toml version and notes.");
    println!("  3. cargo check -p conformance-{name}");
    println!("  4. Add behavioral tests to src/lib.rs.");
}
