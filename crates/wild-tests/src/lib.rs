// Copyright 2026

//! Wild-spec corpus harness.
//!
//! Runs progenitor end-to-end against popular real-world OpenAPI 3.0 and
//! 3.1 documents listed in `manifest.toml`. Each spec is downloaded once
//! into `cache/` (gitignored) and then checked through the pipeline:
//! parse → generate → syntactic validation of the emitted Rust. The
//! manifest records the expected outcome per spec, so the suite acts as a
//! ratchet: regressions fail, and so do silent improvements — flip the
//! manifest entry to lock progress in.

use std::collections::BTreeMap;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// One corpus entry from `manifest.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct SpecEntry {
    pub name: String,
    pub url: String,
    /// Spec version family, informational: "3.0" or "3.1".
    pub version: String,
    /// Expected pipeline outcome.
    pub expect: Expectation,
    /// Large documents that take minutes to generate; only run when
    /// `WILD_SLOW=1`.
    #[serde(default)]
    pub slow: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Expectation {
    /// Parses, generates, and the output is valid Rust.
    Pass,
    /// The document fails to parse or lower.
    ParseFail,
    /// Lowering succeeds but code generation fails (or panics).
    GenerateFail,
    /// Generation succeeds but emits syntactically invalid Rust.
    InvalidRust,
}

#[derive(Debug)]
pub enum Outcome {
    Pass,
    ParseFail(String),
    GenerateFail(String),
    InvalidRust(String),
    /// Not cached and could not be downloaded (offline?); not a verdict.
    Unavailable(String),
}

impl Outcome {
    pub fn matches(&self, expect: Expectation) -> bool {
        matches!(
            (self, expect),
            (Outcome::Pass, Expectation::Pass)
                | (Outcome::ParseFail(_), Expectation::ParseFail)
                | (Outcome::GenerateFail(_), Expectation::GenerateFail)
                | (Outcome::InvalidRust(_), Expectation::InvalidRust)
        )
    }

    pub fn label(&self) -> &'static str {
        match self {
            Outcome::Pass => "pass",
            Outcome::ParseFail(_) => "parse-fail",
            Outcome::GenerateFail(_) => "generate-fail",
            Outcome::InvalidRust(_) => "invalid-rust",
            Outcome::Unavailable(_) => "unavailable",
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            Outcome::Pass => None,
            Outcome::ParseFail(detail)
            | Outcome::GenerateFail(detail)
            | Outcome::InvalidRust(detail)
            | Outcome::Unavailable(detail) => Some(detail),
        }
    }
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(default, rename = "spec")]
    specs: Vec<SpecEntry>,
}

pub fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn load_manifest() -> Result<Vec<SpecEntry>> {
    let path = manifest_dir().join("manifest.toml");
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let manifest: Manifest = toml::from_str(&text).context("parsing manifest.toml")?;

    let mut seen = BTreeMap::new();
    for spec in &manifest.specs {
        if let Some(previous) = seen.insert(spec.name.clone(), &spec.url) {
            anyhow::bail!(
                "duplicate manifest entry `{}` ({} and {})",
                spec.name,
                previous,
                spec.url
            );
        }
    }
    Ok(manifest.specs)
}

pub fn cache_path(entry: &SpecEntry) -> PathBuf {
    let extension = if entry.url.ends_with(".yaml") || entry.url.ends_with(".yml") {
        "yaml"
    } else {
        "json"
    };
    manifest_dir()
        .join("cache")
        .join(format!("{}.{extension}", entry.name))
}

/// Fetch a spec document, preferring the on-disk cache. Set
/// `WILD_REFRESH=1` to re-download.
pub fn fetch(entry: &SpecEntry) -> std::result::Result<String, String> {
    let path = cache_path(entry);
    let refresh = std::env::var_os("WILD_REFRESH").is_some_and(|v| v == "1");
    if !refresh && path.exists() {
        return std::fs::read_to_string(&path).map_err(|err| err.to_string());
    }

    let body = download(&entry.url)?;
    // Reject HTML error pages and other obvious non-spec payloads early.
    let trimmed = body.trim_start();
    if trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<html") {
        return Err(format!("{} returned an HTML page", entry.url));
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, &body).map_err(|err| err.to_string())?;
    Ok(body)
}

fn download(url: &str) -> std::result::Result<String, String> {
    let mut response = ureq::get(url)
        .config()
        .timeout_global(Some(std::time::Duration::from_secs(120)))
        .build()
        .call()
        .map_err(|err| format!("GET {url}: {err}"))?;
    response
        .body_mut()
        .with_config()
        // Wild specs get big (api.github.com is ~4 MB; some are larger).
        .limit(256 * 1024 * 1024)
        .read_to_string()
        .map_err(|err| format!("reading {url}: {err}"))
}

/// Run the full pipeline on a spec document.
///
/// Both parsing/lowering and generation still contain hard panics
/// (todo!/assert/unwrap) on unsupported constructs; catch them so one bad
/// spec can't take the whole corpus run down.
pub fn check_document(text: &str) -> Outcome {
    let parsed = match std::panic::catch_unwind(|| progenitor_impl::parse_openapi_str(text)) {
        Ok(Ok(parsed)) => parsed,
        Ok(Err(err)) => return Outcome::ParseFail(err.to_string()),
        Err(panic) => return Outcome::ParseFail(format!("panic: {}", panic_message(&panic))),
    };

    let generated = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let mut generator = progenitor_impl::Generator::default();
        generator.generate_tokens(&parsed)
    }));

    let tokens = match generated {
        Ok(Ok(tokens)) => tokens,
        Ok(Err(err)) => return Outcome::GenerateFail(err.to_string()),
        Err(panic) => return Outcome::GenerateFail(format!("panic: {}", panic_message(&panic))),
    };

    match syn::parse2::<syn::File>(tokens) {
        Ok(_) => Outcome::Pass,
        Err(err) => Outcome::InvalidRust(err.to_string()),
    }
}

pub fn check_entry(entry: &SpecEntry) -> Outcome {
    match fetch(entry) {
        Ok(text) => check_document(&text),
        Err(err) => Outcome::Unavailable(err),
    }
}

fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = panic.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = panic.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

/// Write a spec's generated client to a standalone crate dir for
/// compile-checking (tier 2, `WILD_COMPILE=1`).
pub fn write_compile_crate(entry: &SpecEntry, out_root: &Path) -> Result<PathBuf> {
    let text = fetch(entry).map_err(anyhow::Error::msg)?;
    let parsed = progenitor_impl::parse_openapi_str(&text)?;
    let mut generator = progenitor_impl::Generator::default();
    let tokens = generator.generate_tokens(&parsed)?;

    let root = out_root.join(&entry.name);
    let src = root.join("src");
    std::fs::create_dir_all(&src)?;

    let mut lib_source = "mod progenitor_client;\n\n".to_string();
    lib_source.push_str(&tokens.to_string());
    std::fs::write(src.join("lib.rs"), lib_source)?;
    std::fs::write(src.join("progenitor_client.rs"), progenitor_client_code())?;
    std::fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"[package]
name = "wild-{name}"
version = "0.0.0"
edition = "2021"

[dependencies]
bytes = "1"
chrono = {{ version = "0.4", default-features = false, features = ["serde"] }}
futures-core = "0.3"
percent-encoding = "2.3"
regress = "0.11"
reqwest = {{ version = "0.13", default-features = false, features = ["json", "query", "stream"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
serde_urlencoded = "0.7"
uuid = {{ version = "1", features = ["serde"] }}

[workspace]
"#,
            name = entry.name
        ),
    )?;
    Ok(root)
}

fn progenitor_client_code() -> &'static str {
    // The same embedded client source cargo-progenitor ships.
    include_str!("../../../progenitor-client/src/progenitor_client.rs")
}
