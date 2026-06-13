// Copyright 2026

//! Conformance suite support crate.
//!
//! Two faces:
//!
//! **Build face** — call from `[build-dependencies]` in every spec crate's
//! build.rs:
//!
//! ```rust,ignore
//! fn main() {
//!     conformance_support::generate("spec.toml");
//! }
//! ```
//!
//! **Test face** — import from `[dev-dependencies]` for the T2 assertion
//! macros (`roundtrip!`, `assert_off_wire!`, `assert_wire_enum!`,
//! `assert_union_variants!`, `assert_rejects!`) and the `serde_json` re-export.

pub use serde_json;

mod fetch;

pub use fetch::SpecManifest;

use std::path::{Path, PathBuf};

/// Root of the conformance workspace (one level above this crate).
fn conformance_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("support crate has a parent dir")
        .to_owned()
}

/// Drive build.rs generation from an explicit spec manifest.
///
/// Reads `spec_manifest` relative to the crate root (cargo sets cwd to the
/// package root for build scripts). Parses `spec.toml`, asserts that the
/// `name` field matches the crate directory name, fetches or retrieves the
/// spec from cache, runs [`progenitor_impl::Generator::generate_text`], and
/// writes `$OUT_DIR/codegen.rs`.
///
/// Set `CONFORMANCE_REFRESH=1` to force re-download even when cached.
///
/// Panics with an actionable message on any failure — appropriate for
/// build.rs where a panic is surfaced as a clear build error.
pub fn generate(spec_manifest: impl AsRef<Path>) {
    let manifest_path = spec_manifest.as_ref();
    let text = std::fs::read_to_string(manifest_path).unwrap_or_else(|err| {
        panic!(
            "conformance_support::generate: cannot read {}: {err}",
            manifest_path.display()
        )
    });

    let manifest: SpecManifest = toml::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "conformance_support::generate: {} is not valid spec.toml: {err}",
            manifest_path.display()
        )
    });

    // Assert that the name in spec.toml matches the crate directory name.
    // This catches copy-paste errors where the name field was not updated.
    let crate_dir_name = std::env::current_dir()
        .expect("cwd")
        .file_name()
        .and_then(|n| n.to_str())
        .expect("cwd has a UTF-8 filename")
        .to_owned();
    assert_eq!(
        manifest.name, crate_dir_name,
        "spec.toml `name = {:?}` must match the crate directory name {:?}; \
         update the name field (copy-paste artifact?)",
        manifest.name, crate_dir_name
    );

    // Tell cargo when to re-run this build script.
    println!("cargo:rerun-if-changed={}", manifest_path.display());
    println!(
        "cargo:rerun-if-env-changed=CONFORMANCE_REFRESH"
    );

    let document = fetch::fetch_spec(&manifest).unwrap_or_else(|err| {
        panic!(
            "conformance_support::generate: cannot fetch spec for {:?}: {err}\n\
             Hint: set CONFORMANCE_REFRESH=1 to force re-download",
            manifest.name,
        )
    });

    let spec = progenitor_impl::parse_openapi_str(&document).unwrap_or_else(|err| {
        panic!(
            "conformance_support::generate: spec {:?} does not parse: {err}",
            manifest.name
        )
    });

    let mut gen = progenitor_impl::Generator::default();
    let generated = gen
        .generate_text(&spec)
        .unwrap_or_else(|err| {
            panic!(
                "conformance_support::generate: code generation failed for {:?}: {err}",
                manifest.name
            )
        });

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR set by cargo"));
    std::fs::write(out_dir.join("codegen.rs"), &generated).unwrap_or_else(|err| {
        panic!("conformance_support::generate: cannot write codegen.rs: {err}")
    });

    let example_tests = generate_example_tests(&gen, &manifest);
    std::fs::write(out_dir.join("example_tests.rs"), &example_tests).unwrap_or_else(|err| {
        panic!("conformance_support::generate: cannot write example_tests.rs: {err}")
    });
}

/// Emit one `#[test]` per (named schema type, example value) found in the
/// generated spec. Returns the contents of `example_tests.rs`; may be empty
/// if the spec has no schema-level examples.
fn generate_example_tests(
    gen: &progenitor_impl::Generator,
    manifest: &fetch::SpecManifest,
) -> String {
    use heck::ToSnakeCase as _;

    let bad: std::collections::HashSet<&str> =
        manifest.bad_examples.iter().map(|s| s.as_str()).collect();

    let mut tests = Vec::new();
    // Track how many times each snake_case base has been used to avoid
    // duplicate function names when two distinct schema names collide in
    // snake_case (e.g. "CountryCode" and "country_code" → "country_code").
    let mut fn_base_count: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();

    for (schema_name, rust_ident, examples) in gen.example_schemas() {
        if bad.contains(schema_name.as_str()) {
            continue;
        }

        let raw_base = schema_name.to_snake_case();
        // Guard against leading digits (rare but possible in wild specs).
        let raw_base = if raw_base.starts_with(|c: char| c.is_ascii_digit()) {
            format!("schema_{raw_base}")
        } else {
            raw_base
        };

        let n = fn_base_count.entry(raw_base.clone()).or_insert(0);
        *n += 1;
        // First occurrence keeps the plain base; later ones get a _v2, _v3, …
        let fn_base = if *n == 1 {
            raw_base
        } else {
            format!("{raw_base}_v{n}")
        };

        for (i, example) in examples.iter().enumerate() {
            let fn_name = format!("example_{fn_base}_{i}");
            let raw_json = serde_json::to_string(example)
                .expect("schema example must be serializable to JSON");
            tests.push(format!(
                "#[test]\nfn {fn_name}() {{\n    \
                 let raw = r###\"{raw_json}\"###;\n    \
                 let json: ::serde_json::Value = ::serde_json::from_str(raw)\n        \
                 .expect(\"built-in spec example must be valid JSON\");\n    \
                 let _ = conformance_support::roundtrip!(crate::types::{rust_ident}, json);\n\
                 }}\n"
            ));
        }
    }

    tests.join("\n")
}

/// Enumerate all spec crates in the conformance workspace by globbing
/// `conformance/*/spec.toml` (excluding the support crate itself).
pub fn all_spec_manifests() -> Vec<(PathBuf, SpecManifest)> {
    let root = conformance_root();
    let mut result = Vec::new();
    let entries = std::fs::read_dir(&root)
        .unwrap_or_else(|err| panic!("cannot read conformance root {}: {err}", root.display()));
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let spec_toml = path.join("spec.toml");
        if !spec_toml.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&spec_toml).unwrap_or_else(|err| {
            panic!("cannot read {}: {err}", spec_toml.display())
        });
        let manifest: SpecManifest = toml::from_str(&text).unwrap_or_else(|err| {
            panic!("{} is not valid spec.toml: {err}", spec_toml.display())
        });
        result.push((path, manifest));
    }
    result.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    result
}

/// Deserialize → serialize → deserialize and assert the two deserialized
/// values are `PartialEq`. Returns the first deserialized value for
/// further assertions.
///
/// ```rust,ignore
/// let val = roundtrip!(MyType, serde_json::json!({"key": "value"}));
/// ```
#[macro_export]
macro_rules! roundtrip {
    ($ty:ty, $json:expr) => {{
        let json: $crate::serde_json::Value = $json;
        let first: $ty = $crate::serde_json::from_value(json.clone())
            .expect(concat!("first deserialization of ", stringify!($ty), " failed"));
        let serialized = $crate::serde_json::to_value(&first)
            .expect(concat!("serialization of ", stringify!($ty), " failed"));
        let second: $ty = $crate::serde_json::from_value(serialized)
            .expect(concat!("second deserialization of ", stringify!($ty), " failed"));
        assert_eq!(
            $crate::serde_json::to_value(&first).unwrap(),
            $crate::serde_json::to_value(&second).unwrap(),
            concat!(stringify!($ty), " round-trip is not idempotent")
        );
        first
    }};
}

/// Assert that none of the given field names appear in the serialized JSON of
/// `$value`. Use to verify that unset `Option` fields stay off the wire.
///
/// ```rust,ignore
/// assert_off_wire!(value, "field_a", "field_b");
/// ```
#[macro_export]
macro_rules! assert_off_wire {
    ($value:expr, $($field:literal),+ $(,)?) => {{
        let serialized = $crate::serde_json::to_value(&$value)
            .expect("serialization failed in assert_off_wire");
        $(
            assert!(
                serialized.get($field).is_none(),
                "field {:?} must be absent from wire representation but was present: {}",
                $field,
                serialized
            );
        )+
    }};
}

/// Assert wire string ↔ variant ↔ Display/FromStr agreement for an enum.
///
/// ```rust,ignore
/// assert_wire_enum!(MyEnum, "variant-a" => MyEnum::VariantA, "b" => MyEnum::B);
/// ```
#[macro_export]
macro_rules! assert_wire_enum {
    ($ty:ty, $($wire:literal => $variant:expr),+ $(,)?) => {{
        $(
            let deserialized: $ty = $crate::serde_json::from_value(
                $crate::serde_json::Value::String($wire.to_string())
            )
            .expect(concat!("deserialization from wire string failed for ", stringify!($ty)));
            assert_eq!(
                deserialized, $variant,
                concat!("wire {:?} should deserialize to ", stringify!($variant)),
                $wire
            );
            let reserialized = $crate::serde_json::to_value(&deserialized)
                .expect("serialization failed");
            assert_eq!(
                reserialized,
                $crate::serde_json::Value::String($wire.to_string()),
                "round-trip serialization mismatch for {:?}",
                $wire
            );
        )+
    }};
}

/// Assert that each JSON value deserializes to the expected union variant.
///
/// ```rust,ignore
/// assert_union_variants!(MyUnion, json_a => MyUnion::VariantA, json_b => MyUnion::VariantB);
/// ```
#[macro_export]
macro_rules! assert_union_variants {
    ($ty:ty, $($json:expr => $variant:pat),+ $(,)?) => {{
        $(
            let deserialized: $ty = $crate::serde_json::from_value($json)
                .expect(concat!("deserialization to ", stringify!($ty), " failed"));
            assert!(
                matches!(deserialized, $variant),
                "expected variant {} but value did not match",
                stringify!($variant),
            );
        )+
    }};
}

/// Assert that deserializing a JSON value into `$ty` fails.
///
/// ```rust,ignore
/// assert_rejects!(MyType, serde_json::json!({"bad": "data"}), "expected reason substr");
/// ```
#[macro_export]
macro_rules! assert_rejects {
    ($ty:ty, $json:expr, $reason:literal) => {{
        let result: Result<$ty, _> = $crate::serde_json::from_value($json);
        assert!(
            result.is_err(),
            concat!(
                "expected deserialization of ",
                stringify!($ty),
                " to fail (",
                $reason,
                ") but it succeeded"
            )
        );
    }};
}

/// Assert at compile time that `$ty` implements `std::fmt::Display`.
///
/// Progenitor uses `Display` (via `to_string()`) to encode path parameters;
/// this assertion turns a silent degradation into a compile error.
///
/// ```rust,ignore
/// assert_display!(types::AccountSid);
/// assert_display!(types::MessageStatus);
/// ```
#[macro_export]
macro_rules! assert_display {
    ($($ty:ty),+ $(,)?) => {
        const _: () = {
            fn _check<T: ::std::fmt::Display>() {}
            $( let _ = _check::<$ty>; )+
        };
    };
}

/// Assert at compile time that `$ty` implements `std::str::FromStr`.
///
/// Progenitor uses `FromStr` for CLI argument parsing; this assertion
/// ensures that path-relevant or CLI-relevant types remain parseable.
///
/// ```rust,ignore
/// assert_from_str!(types::MessageEnumDirection);
/// ```
#[macro_export]
macro_rules! assert_from_str {
    ($($ty:ty),+ $(,)?) => {
        const _: () = {
            fn _check<T: ::std::str::FromStr>() {}
            $( let _ = _check::<$ty>; )+
        };
    };
}

/// Assert at compile time that `$ty` implements `Send + Sync`.
///
/// All generated `Client` types must be `Send + Sync` to be safely shared
/// across threads (e.g., wrapped in `Arc`).
///
/// ```rust,ignore
/// assert_send_sync!(Client);
/// ```
#[macro_export]
macro_rules! assert_send_sync {
    ($($ty:ty),+ $(,)?) => {
        const _: () = {
            fn _check<T: Send + Sync>() {}
            $( let _ = _check::<$ty>; )+
        };
    };
}
