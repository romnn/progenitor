//! The macro accepts YAML documents.

mod load_yaml {
    progenitor::generate_api!("../../sample_openapi/param-overrides.yaml");

    fn _ignore() {
        drop(Client::new("").key_get(None, None));
    }
}
