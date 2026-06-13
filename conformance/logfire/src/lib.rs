include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod example_tests {
    include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
}
