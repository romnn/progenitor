// Copyright 2026

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Parsed contents of a crate's `spec.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct SpecManifest {
    /// Spec name — must match the crate directory name.
    pub name: String,
    /// Remote URL for the spec (mutually exclusive with `file`).
    pub url: Option<String>,
    /// Local path to a committed spec file, relative to the crate root
    /// (mutually exclusive with `url`). Use for hermetic crates such as
    /// petstore-31 that pin a specific spec file in the repo.
    pub file: Option<String>,
    /// OpenAPI version family, informational ("3.0" or "3.1").
    #[serde(default)]
    pub version: Option<String>,
    /// Optional sha256 hex digest for the cached file; reserved for D4
    /// pinning — not yet enforced.
    #[serde(default)]
    pub sha256: Option<String>,
    /// Human-readable notes about this spec (quirks, sources, etc.).
    #[serde(default)]
    pub notes: Option<String>,
    /// Example selectors to skip in T1 auto-generated round-trips.
    #[serde(default)]
    pub bad_examples: Vec<String>,
}

impl SpecManifest {
    pub fn source_url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Fetch the spec document, using `cache_root` as the on-disk cache.
    pub fn fetch_document_with_cache(&self, cache_root: &std::path::Path) -> anyhow::Result<String> {
        fetch_spec_with_cache(self, cache_root)
    }

    /// Fetch the spec document for use in a build script (`OUT_DIR` as cache root).
    pub fn fetch_document(&self) -> anyhow::Result<String> {
        fetch_spec(self)
    }
}

/// Compute the spec file path within `cache_root` for a spec manifest.
pub fn cache_path_for(manifest: &SpecManifest, cache_root: &std::path::Path) -> PathBuf {
    let extension = match &manifest.url {
        Some(url) if url.ends_with(".yaml") || url.ends_with(".yml") => "yaml",
        _ => "json",
    };
    cache_root.join(format!("{}.{extension}", manifest.name))
}

/// Returns the spec cache root for build-script contexts (`$OUT_DIR`).
///
/// Panics if `OUT_DIR` is not set — this is only valid inside a build script.
pub fn build_script_cache_root() -> PathBuf {
    let out_dir = std::env::var_os("OUT_DIR")
        .expect("OUT_DIR not set — build_script_cache_root is only valid in build scripts");
    PathBuf::from(out_dir)
}

/// Fetch the spec document text, preferring the on-disk cache at `cache_root`.
///
/// For `file =` specs, reads the file relative to the current directory
/// (which cargo sets to the crate root for build scripts).
///
/// For `url =` specs, checks `cache_root` first; downloads on cache miss.
/// Set `CONFORMANCE_REFRESH=1` to force re-download.
pub fn fetch_spec_with_cache(manifest: &SpecManifest, cache_root: &std::path::Path) -> Result<String> {
    if let Some(file) = &manifest.file {
        let path = std::path::Path::new(file);
        return std::fs::read_to_string(path)
            .with_context(|| format!("reading local spec file {}", path.display()));
    }

    let url = manifest
        .url
        .as_deref()
        .with_context(|| format!("spec {:?} has neither `url` nor `file`", manifest.name))?;

    let cache_path = cache_path_for(manifest, cache_root);
    let refresh = std::env::var_os("CONFORMANCE_REFRESH").is_some_and(|v| v == "1");

    if !refresh && cache_path.exists() {
        return std::fs::read_to_string(&cache_path)
            .with_context(|| format!("reading cache {}", cache_path.display()));
    }

    let body = download(url).with_context(|| format!("downloading {url}"))?;

    // Reject HTML error pages and other obvious non-spec payloads early.
    let trimmed = body.trim_start();
    if trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<html") {
        anyhow::bail!("{url} returned an HTML page instead of a spec document");
    }

    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating cache dir {}", parent.display()))?;
    }
    std::fs::write(&cache_path, &body)
        .with_context(|| format!("writing cache {}", cache_path.display()))?;

    Ok(body)
}

/// Fetch the spec document text for use in a build script (`OUT_DIR` as cache root).
///
/// Set `CONFORMANCE_REFRESH=1` to force re-download.
pub fn fetch_spec(manifest: &SpecManifest) -> Result<String> {
    fetch_spec_with_cache(manifest, &build_script_cache_root())
}

fn download(url: &str) -> Result<String> {
    // api.weather.gov and some other public APIs reject requests without a
    // User-Agent header. Be honest about what we are.
    let ua = concat!(
        "progenitor-conformance/",
        env!("CARGO_PKG_VERSION"),
        " (+https://github.com/romnn/progenitor)",
    );
    let mut response = ureq::get(url)
        .header("User-Agent", ua)
        .config()
        .timeout_global(Some(std::time::Duration::from_secs(120)))
        .build()
        .call()
        .with_context(|| format!("GET {url}"))?;

    response
        .body_mut()
        .with_config()
        // Wild specs get big (api.github.com is ~4 MB; some are larger).
        .limit(256 * 1024 * 1024)
        .read_to_string()
        .with_context(|| format!("reading response body from {url}"))
}
