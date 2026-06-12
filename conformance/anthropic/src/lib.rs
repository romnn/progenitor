//! Generated Anthropic API client — conformance crate.
//!
//! Demonstrates using progenitor with the Anthropic API and pins, via
//! the tests below, properties of the generated code that a consumer
//! relies on.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let client = Client::new("https://api.anthropic.com");
        assert_eq!(client.baseurl(), "https://api.anthropic.com");
    }
}
