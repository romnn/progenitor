include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod example_tests {
    include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
}

/// Generated axum server stub: the service trait and its `{Api}Server` adapter.
#[allow(clippy::all)]
pub mod server {
    include!(concat!(env!("OUT_DIR"), "/server.rs"));
}
