//! Generate the Discord client at build time from the corpus-cached
//! spec (downloaded on first use by the wild-tests fetch helper).

/// Discord writes discriminant properties as
/// `{"type": "integer", "enum": [N], "allOf": [{"$ref": ...EnumType}]}`
/// (151 occurrences). Typify validates inline `enum` values against the
/// allOf-merged type entry, which at that point can still be an unresolved
/// `Reference` — and `validate_value` rejects references unconditionally
/// (a known TODO in typify's convert.rs), failing the whole generation
/// with `TypeError(InvalidValue)`.
///
/// The inline `type` + `enum` already fully constrains these values, so
/// dropping the redundant `allOf` ref is wire-shape preserving. For the
/// two degenerate `"enum": []` occurrences the ref *is* the real type, so
/// drop the empty enum instead. Remove this once typify validates enum
/// values after reference resolution.
fn strip_redundant_discriminant_refs(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            let empty_enum = map.get("enum").and_then(|e| e.as_array()).is_some_and(|e| e.is_empty());
            if empty_enum {
                map.remove("enum");
            } else if map.contains_key("enum") && map.contains_key("allOf") && map.contains_key("type") {
                map.remove("allOf");
            }
            for v in map.values_mut() {
                strip_redundant_discriminant_refs(v);
            }
        }
        serde_json::Value::Array(values) => {
            for v in values {
                strip_redundant_discriminant_refs(v);
            }
        }
        _ => (),
    }
}

fn main() {
    let entry = wild_tests::load_manifest()
        .expect("corpus manifest loads")
        .into_iter()
        .find(|entry| entry.name == "discord")
        .expect("discord is in the corpus manifest");
    println!(
        "cargo:rerun-if-changed={}",
        wild_tests::cache_path(&entry).display()
    );

    let document = wild_tests::fetch(&entry).expect("spec available");
    let mut json: serde_json::Value = serde_json::from_str(&document).expect("spec is JSON");
    strip_redundant_discriminant_refs(&mut json);
    let document = serde_json::to_string(&json).expect("spec re-serializes");
    let spec = progenitor::parse_openapi_str(&document).expect("spec parses");
    let tokens = progenitor::Generator::default()
        .generate_tokens(&spec)
        .expect("client generates");

    // Build-time assertions about the generated code: it must be a
    // parseable Rust file (syn is the arbiter, not string matching).
    let file = syn::parse2::<syn::File>(tokens).expect("generated client parses as Rust");

    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("codegen.rs");
    std::fs::write(out, prettyplease::unparse(&file)).unwrap();
}
