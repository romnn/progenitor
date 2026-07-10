// Copyright 2025 Oxide Computer Company

use std::path::{Path, PathBuf};

use progenitor_impl::{
    GenerationSettings, Generator, InterfaceStyle, TagStyle, TypeImpl, TypePatch, space_out_items,
};

use proc_macro2::TokenStream;
use progenitor_impl::OpenApiDocument;

fn load_api<P>(p: P) -> OpenApiDocument
where
    P: AsRef<Path> + std::clone::Clone + std::fmt::Debug,
{
    let document = std::fs::read_to_string(p).unwrap();
    progenitor_impl::parse_openapi_str(&document).unwrap()
}

fn generate_formatted(generator: &mut Generator, spec: &OpenApiDocument) -> String {
    let content = generator.generate_tokens(spec).unwrap();
    reformat_code(content)
}

fn reformat_code(content: TokenStream) -> String {
    let rustfmt_config = rustfmt_wrapper::config::Config {
        format_strings: Some(true),
        normalize_doc_attributes: Some(true),
        wrap_comments: Some(true),
        ..Default::default()
    };
    space_out_items(rustfmt_wrapper::rustfmt_config(rustfmt_config, content).unwrap()).unwrap()
}

#[track_caller]
fn verify_apis(openapi_file: &str) {
    let mut in_path = PathBuf::from("../../sample_openapi");
    in_path.push(openapi_file);
    let openapi_stem = openapi_file.split('.').next().unwrap().replace('-', "_");

    let spec = load_api(in_path);

    // Positional generation.
    let mut generator = Generator::default();
    let output = generate_formatted(&mut generator, &spec);
    expectorate::assert_contents(
        format!("tests/output/src/{openapi_stem}_positional.rs"),
        &output,
    );

    // Builder generation with derives and patches.
    let mut generator = Generator::new(
        GenerationSettings::default()
            .with_interface(InterfaceStyle::Builder)
            .with_tag(TagStyle::Merged)
            .with_derive("schemars::JsonSchema")
            .with_patch("Name", TypePatch::default().with_derive("Hash"))
            .with_conversion(
                schemars::schema::SchemaObject {
                    instance_type: Some(schemars::schema::InstanceType::Integer.into()),
                    format: Some("int32".to_string()),
                    ..Default::default()
                },
                "usize",
                [TypeImpl::Display].into_iter(),
            ),
    );
    let output = generate_formatted(&mut generator, &spec);
    expectorate::assert_contents(
        format!("tests/output/src/{openapi_stem}_builder.rs"),
        &output,
    );

    // Builder generation with tags.
    let mut generator = Generator::new(
        GenerationSettings::default()
            .with_interface(InterfaceStyle::Builder)
            .with_cli_bounds("std::clone::Clone")
            .with_tag(TagStyle::Separate),
    );
    let output = generate_formatted(&mut generator, &spec);
    expectorate::assert_contents(
        format!("tests/output/src/{openapi_stem}_builder_tagged.rs"),
        &output,
    );

    // CLI generation.
    let tokens = generator
        .cli(&spec, &format!("crate::{openapi_stem}_builder"))
        .unwrap();
    let output = reformat_code(tokens);

    expectorate::assert_contents(format!("tests/output/src/{openapi_stem}_cli.rs"), &output);

    // httpmock generation.
    let code = generator
        .httpmock(&spec, &format!("crate::{openapi_stem}_builder"))
        .unwrap();

    // TODO pending #368
    let output = rustfmt_wrapper::rustfmt_config(
        rustfmt_wrapper::config::Config {
            format_strings: Some(true),
            ..Default::default()
        },
        code,
    )
    .unwrap();

    let output = progenitor_impl::space_out_items(output).unwrap();
    expectorate::assert_contents(
        format!("tests/output/src/{openapi_stem}_httpmock.rs"),
        &output,
    );
}

#[test]
fn test_keeper() {
    verify_apis("keeper.json");
}

#[test]
fn test_buildomat() {
    verify_apis("buildomat.json");
}

#[test]
fn test_nexus() {
    verify_apis("nexus.json");
}

#[test]
fn test_propolis_server() {
    verify_apis("propolis-server.json");
}

#[test]
fn test_param_override() {
    verify_apis("param-overrides.json");
}

#[test]
fn test_yaml() {
    verify_apis("param-overrides.yaml");
}

#[test]
fn test_param_collision() {
    verify_apis("param-collision.json");
}

#[test]
fn test_cli_gen() {
    verify_apis("cli-gen.json");
}

#[test]
fn test_nexus_with_different_timeout() {
    const OPENAPI_FILE: &str = "nexus.json";

    let mut in_path = PathBuf::from("../../sample_openapi");
    in_path.push(OPENAPI_FILE);
    let openapi_stem = OPENAPI_FILE.split('.').next().unwrap().replace('-', "_");

    let spec = load_api(in_path);

    let mut generator = Generator::new(GenerationSettings::default().with_timeout(75));
    let output = generate_formatted(&mut generator, &spec);
    expectorate::assert_contents(
        format!("tests/output/src/{openapi_stem}_with_timeout.rs"),
        &output,
    );
}

// Server-stub generation golden. Emits the client (for its `types` module) and
// the server module body that references it, exercising response shapes the
// petstore round-trip doesn't: a JSON body, a `201`/no-content (`None`) success,
// a raw octet-stream response, an optional header, a typed default error, and an
// upgrade op that becomes a `501` route stub (no trait method).
#[test]
fn test_server_gen() {
    let spec = load_api("../../sample_openapi/server-gen.json");

    // The client golden provides the `types` module the server golden imports.
    let mut generator = Generator::default();
    let client = generate_formatted(&mut generator, &spec);
    expectorate::assert_contents("tests/output/src/server_gen_client.rs", &client);

    let server = generator.server(&spec, "crate::server_gen_client").unwrap();
    let output = rustfmt_wrapper::rustfmt_config(
        rustfmt_wrapper::config::Config {
            format_strings: Some(true),
            ..Default::default()
        },
        server,
    )
    .unwrap();
    let output = space_out_items(output).unwrap();
    expectorate::assert_contents("tests/output/src/server_gen_server.rs", &output);
}

// TODO this file is full of inconsistencies and incorrectly specified types.
// It's an interesting test to consider whether we try to do our best to
// interpret the intent or just fail.
#[ignore = "fixture contains inconsistent and incorrectly specified types"]
#[test]
fn test_github() {
    verify_apis("api.github.com.json");
}
