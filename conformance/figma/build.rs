//! Generate the Figma client at build time from the corpus-cached
//! spec (downloaded on first use by the wild-tests fetch helper).

fn main() {
    let entry = wild_tests::load_manifest()
        .expect("corpus manifest loads")
        .into_iter()
        .find(|entry| entry.name == "figma")
        .expect("figma is in the corpus manifest");
    println!(
        "cargo:rerun-if-changed={}",
        wild_tests::cache_path(&entry).display()
    );

    let document = wild_tests::fetch(&entry).expect("spec available");
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
