//! Generated github-31 API client — conformance crate.
//!
//! This client is one of the corpus monsters (hundreds of thousands of
//! generated lines); building it takes rustc a long time on stable. The
//! nightly parallel frontend helps:
//! `RUSTFLAGS="-Z threads=8" cargo +nightly check -p conformance-github-31`.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let client = Client::new("https://example.invalid");
        assert_eq!(client.baseurl(), "https://example.invalid");
    }
}
